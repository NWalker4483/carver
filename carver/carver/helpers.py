import bpy, bmesh
from mathutils import Vector, Euler
import math
def snap_object_to_origin(object):
    ######## Snap Target to zero
    minz = 1e8
    maxz = -1
    for vertex in object.data.vertices:
        # object vertices are in object space, translate to world space
        v_world = object.matrix_world @ Vector((vertex.co[0],vertex.co[1],vertex.co[2]))
        if v_world[2] < minz:
            minz = v_world[2]
        if v_world[2] > maxz:
            maxz = v_world[2]
    object.location.z = object.location.z - minz
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

def minimize_objects_angular_distance(objects):
    for i in range(1, len(objects)):
        for ax in range(3):
            diff = objects[i][1][ax] - objects[i-1][1][ax]
            diff = diff % (2*math.pi)
            if diff > math.pi:
                diff -= 2*math.pi
            elif diff < -math.pi:
                diff += 2*math.pi
            objects[i][1][ax] = objects[i-1][1][ax] + diff
    return objects

def pointInsideMesh(point, object):
    axes = [ Vector((1,0,0)) , Vector((0,1,0)), Vector((0,0,1))  ]
    outside = False
    for axis in axes:
        orig = point
        count = 0
        while True:
            hit,location,normal,index = object.ray_cast(orig,orig+axis*10000.0)
            if index == -1: break
            count += 1
            orig = location + axis*0.00001
        if count%2 == 0:
            outside = True
            break
    return not outside

def bmesh_copy_from_object(obj, transform=True, triangulate=True, apply_modifiers=False):
    """
    Returns a transformed, triangulated copy of the mesh
    """

    assert(obj.type == 'MESH')

    if apply_modifiers and obj.modifiers:
        me = obj.to_mesh(bpy.context.scene, True, 'PREVIEW', calc_tessface=False)
        bm = bmesh.new()
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    else:
        me = obj.data
        if obj.mode == 'EDIT':
            bm_orig = bmesh.from_edit_mesh(me)
            bm = bm_orig.copy()
        else:
            bm = bmesh.new()
            bm.from_mesh(me)

    # Remove custom data layers to save memory
    for elem in (bm.faces, bm.edges, bm.verts, bm.loops):
        for layers_name in dir(elem.layers):
            if not layers_name.startswith("_"):
                layers = getattr(elem.layers, layers_name)
                for layer_name, layer in layers.items():
                    layers.remove(layer)

    if transform:
        bm.transform(obj.matrix_world)

    if triangulate:
        bmesh.ops.triangulate(bm, faces=bm.faces)

    return bm

def bmesh_check_intersect_objects(obj, obj2):
    # https://blender.stackexchange.com/questions/9073/how-to-check-if-two-meshes-intersect-in-python
    """
    Check if any faces intersect with the other object

    returns a boolean
    """
    assert(obj != obj2)

    # Triangulate
    bm = bmesh_copy_from_object(obj, transform=True, triangulate=True)
    bm2 = bmesh_copy_from_object(obj2, transform=True, triangulate=True)

    # If bm has more edges, use bm2 instead for looping over its edges
    # (so we cast less rays from the simpler object to the more complex object)
    if len(bm.edges) > len(bm2.edges):
        bm2, bm = bm, bm2

    # Create a real mesh (lame!)
    scene = bpy.context.scene
    me_tmp = bpy.data.meshes.new(name="~temp~")
    bm2.to_mesh(me_tmp)
    bm2.free()
    obj_tmp = bpy.data.objects.new(name=me_tmp.name, object_data=me_tmp)
    # scene.objects.link(obj_tmp)
    bpy.context.collection.objects.link(obj_tmp)
    ray_cast = obj_tmp.ray_cast

    intersect = False

    EPS_NORMAL = 0.000001
    EPS_CENTER = 0.01  # should always be bigger

    #for ed in me_tmp.edges:
    for ed in bm.edges:
        v1, v2 = ed.verts

        # setup the edge with an offset
        co_1 = v1.co.copy()
        co_2 = v2.co.copy()
        co_mid = (co_1 + co_2) * 0.5
        no_mid = (v1.normal + v2.normal).normalized() * EPS_NORMAL
        co_1 = co_1.lerp(co_mid, EPS_CENTER) + no_mid
        co_2 = co_2.lerp(co_mid, EPS_CENTER) + no_mid

        success, co, no, index = ray_cast(co_1, (co_2 - co_1).normalized(), distance = ed.calc_length())
        if index != -1:
            intersect = True
            break

    # scene.objects.unlink(obj_tmp)
    bpy.context.collection.objects.unlink(obj_tmp)
    bpy.data.objects.remove(obj_tmp)
    bpy.data.meshes.remove(me_tmp)
    return intersect

def ray_cast_from_to(origin, target, collision_object, mark=True):
    direction = target - origin
    direction.normalize()
    hit,location,normal,face_index = collision_object.ray_cast(origin, direction)
    
#    hit, location, normal, face_index, _, _ = scene.ray_cast(depsgraph, origin, direction)
    return [hit, location, normal, face_index]
