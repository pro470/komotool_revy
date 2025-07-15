use bevy::{ecs::component::ComponentInfo, prelude::*, render::primitives::Aabb};

use rerun::{AsComponents as _, ComponentBatch, external::nohash_hasher::IntMap};

use crate::{RerunLogger, ToRerun, compute_entity_path};

// ---

/// The default [`RerunLogger`]s that are used if no user-defined logger is specified.
///
/// See [`crate::RerunComponentLoggers`] for more information.
///
/// Public so end users can easily inspect what is configured by default.
#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct DefaultRerunComponentLoggers(IntMap<rerun::ComponentName, Option<RerunLogger>>);

// TODO(cmc): DataUi being typed makes aliases uninspectable :(
#[allow(clippy::too_many_lines)]
impl Default for DefaultRerunComponentLoggers {
    fn default() -> Self {
        let mut loggers = IntMap::default();

        loggers.insert(
            "bevy_transform::components::transform::Transform".into(),
            Some(RerunLogger::new_static(&bevy_transform)),
        );
        loggers.insert(
            "bevy_transform::components::global_transform::GlobalTransform".into(),
            Some(RerunLogger::new_static(&bevy_global_transform)),
        );

        loggers.insert(
            "bevy_render::primitives::Aabb".into(),
            Some(RerunLogger::new_static(&bevy_aabb)),
        );

        loggers.insert(
            "bevy_hierarchy::components::ChildOf::ChildOf".into(),
            Some(RerunLogger::new_static(&bevy_child_of)),
        );
        loggers.insert(
            "bevy_hierarchy::components::children::Children".into(),
            Some(RerunLogger::new_static(&bevy_children)),
        );

        loggers.insert("revy::entity_path::RerunEntityPath".into(), None);

        Self(loggers)
    }
}

// ---

// TODO(cmc): all those aliasing reshenanigans should really just be custom archetype names in
// the descriptor, but the viewer won't be ready for that in 0.22.

fn bevy_transform<'w>(
    _world: &'w World,
    _all_entities: &'w QueryState<(Entity, Option<&'w ChildOf>, Option<&'w Name>)>,
    entity: EntityRef<'_>,
    _component: &'w ComponentInfo,
) -> (Option<&'static str>, Vec<rerun::SerializedComponentBatch>) {
    (
        None,
        entity
            .get::<Transform>()
            .into_iter()
            .flat_map(|transform| transform.to_rerun().as_serialized_batches())
            .collect(),
    )
}

fn bevy_global_transform<'w>(
    _world: &'w World,
    _all_entities: &'w QueryState<(Entity, Option<&'w ChildOf>, Option<&'w Name>)>,
    entity: EntityRef<'_>,
    _component: &'w ComponentInfo,
) -> (Option<&'static str>, Vec<rerun::SerializedComponentBatch>) {
    let suffix = None;
    // TODO(cmc): once again the DataUi does the wrong thing... we really need to
    // go typeless.
    let data = entity
        .get::<GlobalTransform>()
        .into_iter()
        .flat_map(|transform| {
            transform
                .to_rerun()
                .as_serialized_batches()
                .into_iter()
                .map(|batch| {
                    let name = batch.descriptor.component_name;
                    batch.with_descriptor_override(rerun::ComponentDescriptor::new(format!(
                        "{name}Global"
                    )))
                })
        })
        .collect();

    (suffix, data)
}

fn bevy_aabb<'w>(
    world: &'w World,
    _all_entities: &'w QueryState<(Entity, Option<&'w ChildOf>, Option<&'w Name>)>,
    entity: EntityRef<'_>,
    _component: &'w ComponentInfo,
) -> (Option<&'static str>, Vec<rerun::SerializedComponentBatch>) {
    let suffix = Some("aabb");
    let batches = entity
        .get::<Aabb>()
        .map(|aabb| {
            rerun::Boxes3D::from_centers_and_half_sizes(
                [aabb.center.to_rerun()],
                [aabb.half_extents.to_rerun()],
            )
        })
        .map(|aabb| {
            if let Some(mat) = entity
                .get::<MeshMaterial2d<ColorMaterial>>()
                .and_then(|handle| world.resource::<Assets<ColorMaterial>>().get(handle))
            {
                aabb.with_colors([mat.color.to_rerun()])
            } else if let Some(mat) = entity
                .get::<MeshMaterial3d<StandardMaterial>>()
                .and_then(|handle| world.resource::<Assets<StandardMaterial>>().get(handle))
            {
                aabb.with_colors([mat.base_color.to_rerun()])
            } else if let Some(sprite) = entity.get::<Sprite>() {
                aabb.with_colors([sprite.color.to_rerun()])
            } else {
                aabb
            }
        })
        .into_iter()
        .flat_map(|aabb| aabb.as_serialized_batches())
        .collect();
    (suffix, batches)
}

fn bevy_child_of<'w>(
    world: &'w World,
    all_entities: &'w QueryState<(Entity, Option<&'w ChildOf>, Option<&'w Name>)>,
    entity: EntityRef<'_>,
    _component: &'w ComponentInfo,
) -> (Option<&'static str>, Vec<rerun::SerializedComponentBatch>) {
    let suffix = None;
    let batches = entity
        .get::<ChildOf>()
        .and_then(|child_of| {
            let childof_entity_path = compute_entity_path(world, all_entities, child_of.parent());
            rerun::components::EntityPath(childof_entity_path.to_string().into())
                .serialized()
                .map(|batch| {
                    batch.with_descriptor_override(rerun::ComponentDescriptor::new("ChildOf"))
                })
        })
        .into_iter()
        .collect();
    (suffix, batches)
}

fn bevy_children<'w>(
    world: &'w World,
    all_entities: &'w QueryState<(Entity, Option<&'w ChildOf>, Option<&'w Name>)>,
    entity: EntityRef<'_>,
    _component: &'w ComponentInfo,
) -> (Option<&'static str>, Vec<rerun::SerializedComponentBatch>) {
    let suffix = None;
    let batches = entity
        .get::<Children>()
        .and_then(|children| {
            let children = children
                .iter()
                .map(|entity_id| {
                    rerun::components::EntityPath(
                        compute_entity_path(world, all_entities, entity_id)
                            .to_string()
                            .into(),
                    )
                })
                .collect::<Vec<_>>();
            children.serialized().map(|batch| {
                batch.with_descriptor_override(rerun::ComponentDescriptor::new("Children"))
            })
        })
        .into_iter()
        .collect();
    (suffix, batches)
}
