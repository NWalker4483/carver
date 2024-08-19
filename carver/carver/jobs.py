import bpy
from mathutils import Vector
from tqdm import tqdm
import logging

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class XYZIJK_Job:
    def __init__(self, target, tool, tasks=[], origin=Vector((0,0,0))):
        if not isinstance(target, bpy.types.Object) or not isinstance(tool, bpy.types.Object):
            raise ValueError("Target and tool must be valid Blender objects")
        
        self.TOOL = tool
        self.TARGET = target
        self.job_origin = origin
        self.tasks = tasks
        self._keypoints_collection = None
        self.id = id(self)
        self.keypoints = []

        try:
            bpy.ops.object.mode_set(mode='OBJECT')
            self._setup_keypoints_collection()
        except Exception as e:
            logger.error(f"Error in XYZIJK_Job initialization: {str(e)}")
            raise

    def _setup_keypoints_collection(self):
        if "Keypoints" not in bpy.data.collections:
            keypoints_collection = bpy.data.collections.new("Keypoints")
            bpy.context.scene.collection.children.link(keypoints_collection)
        else:
            keypoints_collection = bpy.data.collections.get("Keypoints")
        
        if keypoints_collection:
            if keypoints_collection.name in bpy.context.scene.collection.children:
                bpy.context.scene.collection.children.unlink(keypoints_collection)
            bpy.context.scene.collection.children.link(keypoints_collection)
            
            for obj in keypoints_collection.objects:
                bpy.data.objects.remove(obj, do_unlink=True)
        
        self._keypoints_collection = keypoints_collection

    def mark_keypoints(self, mark_length=0.1):
        try:
            verts = [Vector((0, 0, 0)), Vector((0, 0, mark_length))]
            edges = [(0, 1)]
            line_mesh = bpy.data.meshes.new("LineMesh")
            line_mesh.from_pydata(verts, edges, [])
            line_mesh.update()

            for task in tqdm(self.tasks, desc="Marking keypoints"):
                for location, rotation_euler in task.get_keypoints():
                    line_obj = bpy.data.objects.new("Line", line_mesh)
                    line_obj.location =  location
                    line_obj.rotation_euler = rotation_euler
                    self._keypoints_collection.objects.link(line_obj)
            
            logger.info(f"Keypoints marked successfully")
        except Exception as e:
            logger.error(f"Error in marking keypoints: {str(e)}")
            raise

    def animate(self):
        try:
            self.TOOL.animation_data_clear()
            frame_num = 0
            total_keypoints = sum(len(list(task.get_keypoints())) for task in self.tasks)
            
            with tqdm(total=total_keypoints, desc="Animating tool") as pbar:
                for task in self.tasks:
                    for location, rotation_euler in task.get_keypoints():
                        frame_num += 4
                        
                        # Convert relative location to global coordinates
                        global_location = self.TARGET.location + location
                        logger.info((location, global_location))
                        # Set the tool's position and rotation
                        self.TOOL.location = global_location
                        self.TOOL.rotation_euler = rotation_euler
                        
                        # Insert keyframes
                        self.TOOL.keyframe_insert(data_path='location', frame=frame_num)
                        self.TOOL.keyframe_insert(data_path='rotation_euler', frame=frame_num)
                        
                        pbar.update(1)
            
            bpy.context.scene.frame_end = frame_num
            logger.info(f"Tool animation completed with {frame_num} frames")
        except Exception as e:
            logger.error(f"Error in XYZIJK_Job animation: {str(e)}")
            raise

    def build(self):
        if self.TOOL is None or self.TARGET is None or not bpy.data.objects.get(self.TOOL.name) or not bpy.data.objects.get(self.TARGET.name):
            raise ValueError("Tool or target object is no longer valid")
        
        try:
            self.keypoints = []
            for task in tqdm(self.tasks, desc="Building tasks"):
                if hasattr(task, 'build'):
                    task_keypoints = task.build()
                    if task_keypoints:
                        self.keypoints.extend(task_keypoints)
            logger.info(f"Job built successfully with {len(self.keypoints)} keypoints")
        except Exception as e:
            logger.error(f"Error in building job: {str(e)}")
            raise

    def add_task(self, task):
        self.tasks.append(task)
        logger.info(f"Task added. Total tasks: {len(self.tasks)}")

    def get_tasks(self):
        return self.tasks