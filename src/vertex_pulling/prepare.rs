use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use super::draw::{AuxiliaryMeta, TransformsMeta, ViewMeta};
use super::pipeline::CuboidsPipelines;

use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroupDescriptor, BindGroupEntry},
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
    },
};

pub(crate) fn prepare_clipping_planes(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut clipping_plane_uniform: ResMut<UniformBufferOfGpuClippingPlaneRanges>,
) {
    // Values already pushed in extract stage.
    clipping_plane_uniform.write_buffer(&render_device, &render_queue);
}

pub(crate) fn prepare_materials(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut material_uniforms: ResMut<DynamicUniformBufferOfCuboidMaterial>,
) {
    // Values already pushed in extract stage.
    material_uniforms.write_buffer(&render_device, &render_queue);
}

pub(crate) fn prepare_auxiliary_bind_group(
    pipeline: Res<CuboidsPipelines>,
    render_device: Res<RenderDevice>,
    mut aux_meta: ResMut<AuxiliaryMeta>,
    clipping_plane_uniform: Res<UniformBufferOfGpuClippingPlaneRanges>,
    material_uniform: Res<DynamicUniformBufferOfCuboidMaterial>,
) {
    if let (Some(color_binding), Some(planes_binding)) =
        (material_uniform.binding(), clipping_plane_uniform.binding())
    {
        aux_meta.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("auxiliary_bind_group"),
            layout: &pipeline.aux_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: color_binding,
                },
                BindGroupEntry {
                    binding: 1,
                    resource: planes_binding,
                },
            ],
        }));
    }
}

pub(crate) fn prepare_cuboid_transforms(
    pipeline: Res<CuboidsPipelines>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut transforms_meta: ResMut<TransformsMeta>,
    mut transform_uniforms: ResMut<DynamicUniformBufferOfCuboidTransforms>,
) {
    let write_transform_buffer_span =
        bevy::log::info_span!("prepare_cuboids::write_transform_buffer");
    write_transform_buffer_span.in_scope(|| {
        transform_uniforms.write_buffer(&render_device, &render_queue);
    });
    if let Some(transforms_binding) = transform_uniforms.binding() {
        let create_bind_group_span = bevy::log::info_span!("prepare_cuboids::create_bind_group");
        transforms_meta.transform_buffer_bind_group = create_bind_group_span.in_scope(|| {
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("gpu_cuboids_transforms_bind_group"),
                layout: &pipeline.transforms_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: transforms_binding,
                }],
            }))
        });
    } else {
        assert!(transform_uniforms.is_empty());
    }
}

pub(crate) fn prepare_cuboids(
    pipeline: Res<CuboidsPipelines>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut cuboid_buffers: ResMut<CuboidBufferCache>,
) {
    let write_instance_buffer_span =
        bevy::log::info_span!("prepare_cuboids::write_instance_buffer");
    let create_bind_group_span = bevy::log::info_span!("prepare_cuboids::create_bind_group");

    // Write all dirty buffers from the cuboids cache.
    for entry in cuboid_buffers.entries.values_mut() {
        if !entry.dirty {
            assert!(entry.instance_buffer_bind_group.is_some());
            continue;
        }

        write_instance_buffer_span.in_scope(|| {
            entry
                .instance_buffer
                .write_buffer(&render_device, &render_queue);
        });

        entry.instance_buffer_bind_group = create_bind_group_span.in_scope(|| {
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("cuboids_instance_buffer_bind_group"),
                layout: &pipeline.cuboids_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: entry.instance_buffer.binding().unwrap(),
                }],
            }))
        });

        entry.dirty = false;
    }
}

pub(crate) fn prepare_cuboids_view_bind_group(
    render_device: Res<RenderDevice>,
    cuboids_pipeline: Res<CuboidsPipelines>,
    mut view_meta: ResMut<ViewMeta>,
    view_uniforms: Res<ViewUniforms>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        view_meta.cuboids_view_bind_group =
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding,
                }],
                label: Some("cuboids_view_bind_group"),
                layout: &cuboids_pipeline.view_layout,
            }));
    }
}
