import bpy
from mathutils import Vector, Matrix
import sys
from tqdm import tqdm
import logging
import imp
sys.path.append('/home/walkenz1/Projects/merlin_ws/src/extras/carver')
import carver
imp.reload(carver)
imp.reload(carver.cam)
imp.reload(carver.jobs)
from carver.cam import MultiContourTrace
from carver.jobs import XYZIJK_Job

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

def prep_target(target, table_position):
    try:
        bpy.context.view_layer.objects.active = target
        target.select_set(True)
        
        # Make the object single-user
        bpy.ops.object.make_single_user(object=True, obdata=True, material=False, animation=False)
        
        # Simplify Model
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.dissolve_limited()
        bpy.ops.object.mode_set(mode='OBJECT')
        
        # Set origin to center of mass
        bpy.ops.object.origin_set(type='ORIGIN_CENTER_OF_MASS', center='MEDIAN')
        
        # Get the bounds of the target object
        bound_box = [target.matrix_world @ Vector(corner) for corner in target.bound_box]
        min_z = min(v.z for v in bound_box)
        max_z = max(v.z for v in bound_box)
        
        # Calculate the offset to move the origin to the bottom
        z_offset = min_z - target.location.z
        
        # Move the origin to the bottom
        target.data.transform(Matrix.Translation((0, 0, -z_offset)))
        
        # Position the object on the table
        target.location = Vector((table_position.x, table_position.y, table_position.z + .025 + 0.001))
        
        # Apply transformations
        bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
        
        logger.info("Target preparation completed successfully")
    except Exception as e:
        logger.error(f"Error in preparing target: {str(e)}")
        raise

####### SETUP ########
TARGET = bpy.data.objects.get("TARGET")
TOOL = bpy.data.objects.get("tool_control")
if TARGET is None or TOOL is None:
    raise ValueError("TARGET or TOOL object not found in the scene")

####### Cleanup ########
# Remove any leftover animation data
for obj in bpy.data.objects:
    if obj.animation_data:
        obj.animation_data_clear()

# Clear unused data
bpy.ops.outliner.orphans_purge(do_recursive=True)

####### Get table position ########
armature_name = "merlin"
bone_name = "table_1"
armature = bpy.data.objects.get(armature_name)
if armature is None:
    raise ValueError(f"Armature '{armature_name}' not found")

bone = armature.pose.bones.get(bone_name)
if bone is None:
    raise ValueError(f"Bone '{bone_name}' not found in armature '{armature_name}'")

table_position = armature.matrix_world @ bone.head

####### Prepare TARGET ########
prep_target(TARGET, table_position)

# #### Process BlenderCAM
job = XYZIJK_Job(tool=TOOL, target=TARGET)
job.add_task(MultiContourTrace(TARGET, TOOL, start=Vector((0,0,0)), height=.5, cuts=3))
job.build()
# job.mark_keypoints(.05)
job.animate()

logger.info("Script executed successfully")
