import bpy, bmesh
from mathutils import Vector, Euler
import math
import copy 
from tqdm import tqdm
import logging
import sys

sys.path.append('/home/walkenz1/Projects/merlin_ws/src/extras/carver')
from carver.helpers import bmesh_check_intersect_objects, minimize_objects_angular_distance, pointInsideMesh, ray_cast_from_to

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class CAMTask:
    def __init__(self, target, tool):
        if not isinstance(target, bpy.types.Object) or not isinstance(tool, bpy.types.Object):
            raise ValueError("Target and tool must be valid Blender objects")
        self._keypoints = [] 
        self.TARGET = target
        self.TOOL = tool
        self.TOOL.rotation_mode = "XYZ"

    def get_keypoints(self):
        return copy.deepcopy(self._keypoints)

    def build(self):
        try:
            self._build()
            # self.remove_unreachable_keypoints()
            logger.info(f"Built {self.__class__.__name__} with {len(self._keypoints)} keypoints")
        except Exception as e:
            logger.error(f"Error in building {self.__class__.__name__}: {str(e)}")
            raise

    def _build(self):
        # This method should be overridden by subclasses
        raise NotImplementedError("Subclasses must implement the _build method")

    def remove_unreachable_keypoints(self):
        try:
            original_count = len(self._keypoints)
            marks = []
            for location, rotation_euler in tqdm(self._keypoints, desc="Checking reachable keypoints"):
                bpy.context.view_layer.objects.active = self.TOOL
                self.TOOL.location = location
                self.TOOL.rotation_euler = rotation_euler
                bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
                if not bmesh_check_intersect_objects(self.TOOL, self.TARGET):
                    marks.append((location, rotation_euler))
            self._keypoints = marks
            removed_count = original_count - len(self._keypoints)
            logger.info(f"Removed {removed_count} unreachable keypoints")
        except Exception as e:
            logger.error(f"Error in removing unreachable keypoints: {str(e)}")
            raise

class ContourTrace(CAMTask):
    def __init__(self, target, tool, center, plane_normal=Vector((0,0,1))):
        super().__init__(target, tool)
        self.center = center
        self.plane_normal = plane_normal
        self.contour = None
          
    def sample_for_normals(self, offset=0):
        try:
            for vert in tqdm(self.contour.data.vertices, desc="Sampling for normals"):
                point = Vector((vert.co.x, vert.co.y, vert.co.z))
                result = [False]
                for p1, p2 in [((.5,.5,0), (-.5,-.5,0)), ((-.5,.5,0), (.5,-.5,0))]:
                    v1, v2 = point + Vector(p1) * .25, point + Vector(p2) * .25
                    if pointInsideMesh(v1, self.TARGET) and not pointInsideMesh(v2, self.TARGET):
                        result = ray_cast_from_to(v2, v1, self.TARGET)
                        break
                    elif pointInsideMesh(v2, self.TARGET) and not pointInsideMesh(v1, self.TARGET):
                        result = ray_cast_from_to(v1, v2, self.TARGET)
                        break
                if result[0]: 
                    hit, location, normal, face_index = result
                    rotation_euler = normal.to_track_quat('Z', 'Y').to_euler("XYZ")
                    location = location + (normal * offset)
                    self._keypoints.append([location, rotation_euler])
            logger.info(f"Sampled {len(self._keypoints)} normals")
        except Exception as e:
            logger.error(f"Error in sampling for normals: {str(e)}")
            raise

    def get_contour(self):
        try:
            bm = bmesh.new()
            bm.from_mesh(self.TARGET.data)
           
            cut = bmesh.ops.bisect_plane(
                    bm,
                    plane_co=self.center,
                    plane_no=self.plane_normal,
                    geom=bm.verts[:] + bm.faces[:] + bm.edges[:],
                    clear_inner=True,
                    clear_outer=True,
                    )["geom_cut"]
            
            if not cut:
                bm.clear()
                logger.warning("No contour found")
                return
            
            me = bpy.data.meshes.new(f"Slice")
            bm.to_mesh(me)
     
            slice = bpy.data.objects.new(f"Slice", me)
            slice.matrix_world = self.TARGET.matrix_world
            self.contour = slice
            bm.clear()   
            
            slice.modifiers.new(name='Subdivision', type='SUBSURF')
            slice.modifiers["Subdivision"].levels = 4
            slice.modifiers["Subdivision"].subdivision_type = 'SIMPLE'
            bpy.context.scene.collection.objects.link(slice)
            bpy.context.view_layer.objects.active = slice
            bpy.ops.object.modifier_apply(modifier="Subdivision")
            logger.info("Contour created successfully")
        except Exception as e:
            logger.error(f"Error in getting contour: {str(e)}")
            raise
    
    def _build(self):
        self.get_contour()
        if self.contour is not None:
            def get_angle_z(obj):
                location, rotation = obj
                return math.atan2(location.y, location.x)

            self.sample_for_normals(offset=.0)
            self._keypoints = sorted(self._keypoints, key=get_angle_z)
            self._keypoints = minimize_objects_angular_distance(self._keypoints)

class MultiContourTrace(CAMTask):
    def __init__(self, target, tool, start=Vector((0,0,0)), height=10, cuts=1, plane_normal=Vector((0,0,1))):
        super().__init__(target, tool)
        self.start = start
        self.cuts = cuts
        self.sub_tasks = [] 
        self.height = height
        
    def get_keypoints(self):
        points = []
        for task in self.sub_tasks:
            points += task.get_keypoints()
        return points
    
    def _build(self):
        start = self.start
        end = self.start.copy()
        end.z += self.height
        
        axis = end - start
        dv = axis / self.cuts
        
        for i in tqdm(range(self.cuts + 1), desc="Building multi-contour trace"):
            plane_co = self.start + i * dv
            task = ContourTrace(self.TARGET, self.TOOL, center=plane_co, plane_normal=axis)
            task.build()
            self.sub_tasks.append(task)
        logger.info(f"Built multi-contour trace with {len(self.sub_tasks)} sub-tasks")