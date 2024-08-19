
# def introduce_p_axis(raw_job):
#     max_d_theta = 0 # math.pi/4
#     points = [] 
#     angles = []
#     for task in raw_job.tasks.copy():
#         for i,(location, rotation) in enumerate(task.get_keypoints()):   
#             rotation = list(rotation)
#             angle = full_angle_about_origin(location[0], location[1])
#             d_theta = 0 
#             if angle > max_d_theta:
#                 pass
#             d_theta =  angle - max_d_theta
#             location = rotate_z_axis(*location, -d_theta)
#             rotation[2] += -d_theta
            
#             points.append([location, rotation])
#             angles.append(-d_theta)
            
#     points = minimize_objects_angular_distance([points])[0]
                
#     key_task = FollowKeypoints(tool=raw_job.TOOL, target=raw_job.TARGET, keypoints=points)
#     new_job = XYZIJK_Job(tool=raw_job.TOOL, target=raw_job.TARGET, tasks = [key_task])
#     turns = Rotary_Job(tool=raw_job.TOOL, target=raw_job.TARGET, angles=angles)
#     return new_job, turns

        
    def import_gcode(self, path="/Users/walkenz1/Desktop/test.job"):
        with open(path,"r") as out:
            lines= out.readlines()
            points = []
            for line in lines:
                x,y,z,i,j,k = line.split(",")
                location = [float(i.strip()[1:]) for i in [x,y,z]]
                normal = [float(p.strip()[1:]) for p in [i,j,k]]
                
                points.append((location, ijk_to_euler(normal)))
        
        key_task = FollowKeypoints(tool=self.TOOL, target=self.TARGET, keypoints=points)
        self.add_task(key_task)
        
    def export_gcode(self, path="/Users/walkenz1/Desktop/test.job"):
        # Animate and Save
        with open(path,"w") as out:
            for i, task in enumerate(self.tasks):
                for location, rotation_euler in task.get_keypoints():
                    I,J,K = euler_to_ijk(rotation_euler)
                    out.write(f"X{location.x}, Y{location.y}, Z{location.z}, I{I}, J{J}, K{K}\n")
             

from mathutils import Vector, Euler
import math
def rotate_z_axis(x, y, z, radians):
    # Perform the rotation
    x_new = x * math.cos(radians) - y * math.sin(radians)
    y_new = x * math.sin(radians) + y * math.cos(radians)
    z_new = z

    # Return the rotated coordinates
    return x_new, y_new, z_new

def euler_to_ijk(euler_angles):
    x, y, z = euler_angles
    rotation = Euler((x, y, z), 'XYZ')
    i, j, k = rotation
                       
class FollowKeypoints(CAMTask):
    def __init__(self, target, tool, keypoints = []):
        super().__init__(target=target, tool=tool)
        self._keypoints = keypoints
  
class LinearTransition(CAMTask):
    def __init__(self, target, tool):
        super().__init__(target, tool)

class RadialTransition(CAMTask):
    def __init__(self, target, tool):
        super().__init__(target, tool)
class Rotary_Job:
    def __init__(self, target, tool, angles=[]):
        self.TARGET = target
        self.angles = angles

    def animate(self):
        try:
            self.TARGET.animation_data_clear()
            frame_num = 0
            for i, angle in tqdm(enumerate(self.angles), total=len(self.angles), desc="Animating rotation"):
                frame_num += 1
                self.TARGET.rotation_euler.z = angle
                self.TARGET.keyframe_insert(data_path='rotation_euler', frame=frame_num)
            bpy.context.scene.frame_end = frame_num
            logger.info(f"Rotation animation completed with {frame_num} frames")
        except Exception as e:
            logger.error(f"Error in Rotary_Job animation: {str(e)}")
            raise
