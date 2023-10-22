use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Added, Query, Res, ResMut};
use bevy_time::Time;
use tracing::{debug, trace};

use lightyear_shared::replication::{Replicate, ReplicationSend};
use lightyear_shared::Protocol;

use crate::Server;

// TODO: rename io receive ?
// or do additional receive stuff here.
pub(crate) fn receive<P: Protocol>(time: Res<Time>, mut server: ResMut<Server<P>>) {
    trace!("Receive client packets");
    // update client state, send keep-alives, receive packets from io
    server.update(time.elapsed().as_secs_f64()).unwrap();
    // buffer packets into message managers
    server.recv_packets().unwrap();

    // receive events
    // server.receive()
}

// or do additional send stuff here
pub(crate) fn send<P: Protocol>(mut server: ResMut<Server<P>>) {
    trace!("Send packets to clients");
    // finalize any packets that are needed for replication
    server.prepare_replicate_send();
    // send buffered packets to io
    server.send_packets().unwrap();
}

// TODO: on connect event, replicate everything!

pub(crate) fn send_entity_spawn<P: Protocol>(
    mut server: ResMut<Server<P>>,
    // try doing entity spawn whenever replicate gets added (as it could be the first time)
    query: Query<(Entity, &Replicate), Added<Replicate>>,
) {
    // TODO: distinguish between new entity or just replicate got added ?
    //  Maybe by adding an extra component the first time the entity gets created? or a flag in the Replicate component?

    for (entity, replicate) in query.iter() {
        server.entity_spawn(entity, vec![], replicate);
    }
}

// pub(crate) fn send_entity_despawn<P: Protocol>(
//     mut server: ResMut<Server<P>>,
//     mut query: RemovedComponents<DespawnTracker>,
// ) {
//     // TODO: distinguish between new entity or just replicate got added ?
//     //  Maybe by adding an extra component the first time the entity gets created? or a flag in the Replicate component?
//     query.iter().try_for_each(|entity| {
//         server.entity_despawn(entity);
//     });
// }

// TODO: THE OTHER APPROACH WOULD BE TO CREATE INDIVIDUAL SYSTEMS
// REPLICATE_ENTITY_UPDATE<C: COMPONENT>() {
//   Query<Entity, Changed<C>>,
// )

// pub(crate) fn replicate_entity_updates<P: Protocol>(
//     mut server: ResMut<Server<P>>,
//     world: &World,
//     system_ticks: SystemChangeTick,
//     shared: Res<ReplicationData>,
// ) {
//     for archetype in world
//         .archetypes()
//         .iter()
//         .filter(|archetype| archetype.id() != ArchetypeId::EMPTY)
//         .filter(|archetype| archetype.id() != ArchetypeId::INVALID)
//         .filter(|archetype| archetype.contains(shared.replication_id))
//     {
//         let table = world
//             .storages()
//             .tables
//             .get(archetype.table_id())
//             .expect("archetype should be valid");
//         for archetype_entity in archetype.entities() {
//             let entity = archetype_entity.entity();
//             // Cannot error since the archetype contains replicate
//             let replicate = world.entity(entity).get::<Replicate>().unwrap();
//             let mut components: Vec<P::Components> = Vec::new();
//             // TODO: heuristic, if we have more components in protocol, iterate through the entity's components
//             //  else iterate through the protocol's components
//             for component_id in archetype.components() {
//                 if shared.contains_component(component_id) {
//                     let storage_type = archetype
//                         .get_storage_type(component_id)
//                         .unwrap_or_else(|| panic!("{component_id:?} be in archetype"));
//                     match storage_type {
//                         StorageType::Table => {
//                             let column = table.get_column(component_id).unwrap_or_else(|| {
//                                 panic!("{component_id:?} should belong to table")
//                             });
//
//                             // SAFETY: the table row obtained from the world state.
//                             let ticks =
//                                 unsafe { column.get_ticks_unchecked(archetype_entity.table_row()) };
//                             // SAFETY: component obtained from the archetype.
//                             let component =
//                                 unsafe { column.get_data_unchecked(archetype_entity.table_row()) };
//
//                             // TODO: use client last ack?
//                             if ticks.is_changed(system_ticks.last_run(), system_ticks.this_run()) {
//                                 // TODO: this means we can only replicate cloneable components for now
//                                 // TODO: should we store some type-erased function to convert from component-PTR to protocol component?
//                                 unsafe {
//                                     components.push(component.deref().clone().into());
//                                 }
//                             }
//                         }
//                         StorageType::SparseSet => {
//                             let sparse_set = world
//                                 .storages()
//                                 .sparse_sets
//                                 .get(component_id)
//                                 .unwrap_or_else(|| {
//                                     panic!("{component_id:?} should be in sparse set")
//                                 });
//
//                             let entity = archetype_entity.entity();
//                             let ticks = sparse_set.get_ticks(entity).unwrap_or_else(|| {
//                                 panic!("{entity:?} should have {component_id:?}")
//                             });
//                             let component = sparse_set.get(entity).unwrap_or_else(|| {
//                                 panic!("{entity:?} should have {component_id:?}")
//                             });
//                             // TODO: use client last ack?
//                             if ticks.is_changed(system_ticks.last_run(), system_ticks.this_run()) {
//                                 // TODO: this means we can only replicate cloneable components for now
//                                 unsafe {
//                                     components.push(component.deref().clone().into());
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//             server.entity_update(entity, components, replicate);
//         }
//     }
// }
//
// // fn replicate_entity_spawn<P: Protocol>(world: &World) {
// //     world.entity().archetype().table_components()
// // }
// //     for archetype in world
// //         .archetypes()
// //         .iter()
// //         .filter(|archetype| archetype.id() != ArchetypeId::EMPTY)
// //         .filter(|archetype| archetype.id() != ArchetypeId::INVALID)
// //         .filter(|archetype| archetype.contains(replication_rules.get_marker_id()))
// //     {
// // }
