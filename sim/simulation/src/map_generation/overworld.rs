//! Generate high level room layout
//!
mod params;
pub use params::*;

use crate::components::{RoomComponent, RoomConnection, RoomConnections};
use crate::geometry::{Axial, Hexagon};
use crate::prelude::hex_round;
use crate::storage::views::UnsafeView;
use crate::tables::morton_table::{ExtendFailure, MortonTable};
use rand::Rng;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Debug, Clone, Error)]
pub enum OverworldGenerationError {
    #[error("Can not place {number_of_rooms} rooms in an area with radius of {radius}")]
    BadRadius { number_of_rooms: u32, radius: u32 },

    #[error("Failed to build Room table: {0:?}")]
    ExtendFail(ExtendFailure),

    #[error("Failed to build Room weight table: {0:?}")]
    WeightMapInitFail(ExtendFailure),
}

/// Transform the room_id into absolute 'room tile' space
fn room_id_to_axial(room_id: Axial, grid_size: i32) -> Axial {
    const SQRT3: f64 = 1.73205080756887;
    let size = (grid_size as f64 + 1.0) * SQRT3;

    // 1) convert room_id to world space based on flat top hexes with size `grid_size`
    let x = size * (3. / 2. * room_id.q as f64);
    let y = size * (SQRT3 / 2. * room_id.q as f64 + SQRT3 * room_id.r as f64);

    // 2) convert the world space coordinates back to axial, but pointy-top with radius of 1
    let q = SQRT3 / 3. * x - 1. / 3. * y; // /1 omitted
    let r = 2. / 3. * y;

    hex_round([q as f32, r as f32])
}

/// Insert the given number of rooms in the given radius (where the unit is a room).
///
/// [ ] TODO: remove some nodes to produce less dense maps?
/// [ ] TODO: resource map?
/// [ ] TODO: political map?
/// [ ] TODO: parallellism?
/// [ ] TODO: non-uniform room sizes
pub fn generate_room_layout(
    OverworldGenerationParams {
        radius,
        room_radius,
        min_bridge_len,
        max_bridge_len,
    }: &OverworldGenerationParams,
    rng: &mut impl Rng,
    (mut rooms, mut room_connections): (
        UnsafeView<Axial, RoomComponent>,
        UnsafeView<Axial, RoomConnections>,
    ),
) -> Result<(), OverworldGenerationError> {
    let radius = *radius as i32;
    let room_radius = *room_radius as i32;
    let center = Axial::new(radius, radius);
    let bounds = Hexagon { center, radius };

    // Init the grid
    rooms.clear();
    rooms
        .extend(bounds.iter_points().map(|room_id| {
            (
                room_id,
                RoomComponent {
                    offset: room_id_to_axial(room_id, room_radius),
                    seed: rng.gen(),
                },
            )
        }))
        .map_err(OverworldGenerationError::ExtendFail)?;

    room_connections.clear();
    room_connections
        .extend(bounds.iter_points().map(|p| (p, Default::default())))
        .map_err(OverworldGenerationError::ExtendFail)?;

    debug!("Building room_connections");

    // loosely running the Erdos - Runyi model
    let connection_weights = MortonTable::from_iterator(bounds.iter_points().map(|p| {
        let weight = rng.gen_range(-4.0..=6.0);
        let weight = sigmoid(weight);
        (p, weight)
    }))
    .map_err(OverworldGenerationError::WeightMapInitFail)?;

    for point in bounds.iter_points() {
        update_room_connections(
            room_radius as u32,
            *min_bridge_len,
            *max_bridge_len,
            point,
            &connection_weights,
            rng,
            room_connections,
        );
    }
    debug!("Building room_connections done");

    // TODO: insert more room_connections if the graph is not fully connected

    Ok(())
}

fn sigmoid(f: f32) -> f32 {
    1.0 / (1.0 + std::f32::consts::E.powf(-f))
}

fn update_room_connections(
    room_radius: u32,
    min_bridge_len: u32,
    max_bridge_len: u32,
    point: Axial,
    connection_weights: &MortonTable<f32>,
    rng: &mut impl Rng,
    mut room_connections: UnsafeView<Axial, RoomConnections>,
) {
    let w = rng.gen_range(0.0..1.0);
    let mut to_connect = [None; 6];
    connection_weights.query_range(point, 3, &mut |p, weight| {
        if w <= *weight {
            let n = p - point;
            if let Some(i) = Axial::neighbour_index(n) {
                to_connect[i] = Some(n);
            }
        }
    });

    if !to_connect.iter().any(|c| c.is_some()) {
        // if this room has no room_connections insert 1 at random
        let mut weights = [0.0; 6];
        connection_weights.query_range(point, 3, &mut |p, _| {
            let n = p - point;
            if let Some(i) = Axial::neighbour_index(n) {
                weights[i] = rng.gen_range(0.5..=1.0);
            }
        });
        let (i, _) = weights
            .iter()
            .enumerate()
            .max_by(|(_, w1), (_, w2)| w1.partial_cmp(w2).expect("Expected non-nan values"))
            .expect("Expected all rooms to have at least 1 neighbour");

        to_connect[i] = Some(point.hex_neighbours()[i] - point);
    }

    let current_connections = {
        let to_connect = &mut to_connect[..];
        room_connections.update_with(point, |RoomConnections(ref mut conn)| {
            for (i, c) in to_connect.iter_mut().enumerate() {
                if conn[i].is_none() && c.is_some() {
                    let bridge_len = rng.gen_range(min_bridge_len..=max_bridge_len);
                    let padding = room_radius - bridge_len;

                    let offset_start = rng.gen_range(0..padding);
                    let offset_end = padding - offset_start;

                    // this is a new connection
                    conn[i] = c.map(|c| RoomConnection {
                        direction: c,
                        offset_start,
                        offset_end,
                    });
                } else {
                    // if we don't have to update this posision then set it to None so we don't
                    // attempt to update the neighbour later.
                    *c = None;
                }
            }
        })
    }
    .expect("expected the current room to have connection")
    .clone();

    for neighbour in current_connections
        .0
        .iter()
        .filter_map(|n| n.as_ref())
        .cloned()
    {
        room_connections.update_with(point + neighbour.direction, |conn| {
            let inverse = neighbour.direction * -1;
            let i = Axial::neighbour_index(inverse)
                .expect("expected neighbour inverse to be a valid neighbour posision");
            // this one's offsets are the current room's inverse
            let offset_end = neighbour.offset_start;
            let offset_end = offset_end.max(1) - 1; // offset_end - 1 or 0
            let offset_start = neighbour.offset_end + 1;

            conn.0[i] = Some(RoomConnection {
                direction: inverse,
                offset_start,
                offset_end,
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overworld_connections_are_valid() {
        let mut rooms = MortonTable::new();
        let mut room_connections = MortonTable::new();

        let params = OverworldGenerationParams::builder()
            .with_radius(12)
            .with_room_radius(16)
            .with_min_bridge_len(3)
            .with_max_bridge_len(12)
            .build()
            .unwrap();
        generate_room_layout(
            &params,
            &mut rand::thread_rng(),
            (
                UnsafeView::from_table(&mut rooms),
                UnsafeView::from_table(&mut room_connections),
            ),
        )
        .unwrap();

        assert_eq!(rooms.len(), room_connections.len());

        // for each connection of the room test if the corresponding connection of the neighbour
        // is valid.
        for (room, RoomConnections(ref room_conn)) in room_connections.iter() {
            for conn in room_conn.iter().filter_map(|x| x.as_ref()) {
                let RoomConnections(ref conn_pairs) = room_connections
                    .at(room + conn.direction)
                    .expect("Expected the neighbour to be in the room_connections table");

                let i = Axial::neighbour_index(conn.direction * -1).unwrap();
                let conn_pair = conn_pairs[i]
                    .as_ref()
                    .expect("The pair connection was not found");

                assert_eq!(conn_pair.direction, conn.direction * -1);
            }
        }
    }
}
