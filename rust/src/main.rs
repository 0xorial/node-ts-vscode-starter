mod solution;

use crate::solution::{
    get_move, GameState, Inventory, LearnableSpell, Move, Order, Spell, SpellDescriptor,
};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_hc::Hc128Rng;
use std::collections::VecDeque;
use std::iter::FromIterator;

fn make_initial_state<'a>(
    all_potions: &'a [Order],
    all_spells: &'a [SpellDescriptor],
    initial_spells: &'a [SpellDescriptor],
) -> GameState<'a> {
    let mut rng = Hc128Rng::seed_from_u64(42);
    let mut shuffled_spells = Vec::from_iter(all_spells);
    shuffled_spells.shuffle(&mut rng);
    let mut shuffled_spells = VecDeque::from_iter(shuffled_spells);
    let mut learnable_spells = Vec::with_capacity(6);
    for _ in 0..6 {
        let spell = shuffled_spells.pop_front().expect("");
        learnable_spells.push(LearnableSpell {
            descriptor: spell,
            reward: 0,
        });
    }

    let mut shuffled_orders = Vec::from_iter(all_potions);
    shuffled_orders.shuffle(&mut rng);
    let mut shuffled_orders = VecDeque::from_iter(shuffled_orders);
    let mut orders = VecDeque::with_capacity(5);
    for _ in 0..5 {
        let spell = shuffled_orders.pop_front().expect("");
        orders.push_back(spell);
    }

    GameState {
        inventory: [3, 0, 0, 0],
        next_id: 0,

        learnable_spells,
        next_learnable_spells: shuffled_spells,

        orders,
        next_orders: shuffled_orders,

        potions_brewed: 0,
        score: 0,
        moves: 0,
        spells: Vec::from_iter(initial_spells.iter().map(|x| Spell {
            descriptor: x,
            exhausted: false,
        })),
    }
}

static SPELLS_ARRAY: &'static [[i8; 4]] = &[
    [-3, 0, 0, 1],
    [3, -1, 0, 0],
    [1, 1, 0, 0],
    [0, 0, 1, 0],
    [3, 0, 0, 0],
    [2, 3, -2, 0],
    [2, 1, -2, 1],
    [3, 0, 1, -1],
    [3, -2, 1, 0],
    [2, -3, 2, 0],
    [2, 2, 0, -1],
    [-4, 0, 2, 0],
    [2, 1, 0, 0],
    [4, 0, 0, 0],
    [0, 0, 0, 1],
    [0, 2, 0, 0],
    [1, 0, 1, 0],
    [-2, 0, 1, 0],
    [-1, -1, 0, 1],
    [0, 2, -1, 0],
    [2, -2, 0, 1],
    [-3, 1, 1, 0],
    [0, 2, -2, 1],
    [1, -3, 1, 1],
    [0, 3, 0, -1],
    [0, -3, 0, 2],
    [1, 1, 1, -1],
    [1, 2, -1, 0],
    [4, 1, -1, 0],
    [-5, 0, 0, 2],
    [-4, 0, 1, 1],
    [0, 3, 2, -2],
    [1, 1, 3, -2],
    [-5, 0, 3, 0],
    [-2, 0, -1, 2],
    [0, 0, -3, 3],
    [0, -3, 3, 0],
    [-3, 3, 0, 0],
    [-2, 2, 0, 0],
    [0, 0, -2, 2],
    [0, -2, 2, 0],
    [0, 0, 2, -1],
];

static ORDERS_ARRAY: &'static [[i32; 5]] = &[
    [2, 2, 0, 0, 6],
    [3, 2, 0, 0, 7],
    [0, 4, 0, 0, 8],
    [2, 0, 2, 0, 8],
    [2, 3, 0, 0, 8],
    [3, 0, 2, 0, 9],
    [0, 2, 2, 0, 10],
    [0, 5, 0, 0, 10],
    [2, 0, 0, 2, 10],
    [2, 0, 3, 0, 11],
    [3, 0, 0, 2, 11],
    [0, 0, 4, 0, 12],
    [0, 2, 0, 2, 12],
    [0, 3, 2, 0, 12],
    [0, 2, 3, 0, 13],
    [0, 0, 2, 2, 14],
    [0, 3, 0, 2, 14],
    [2, 0, 0, 3, 14],
    [0, 0, 5, 0, 15],
    [0, 0, 0, 4, 16],
    [0, 2, 0, 3, 16],
    [0, 0, 3, 2, 17],
    [0, 0, 2, 3, 18],
    [0, 0, 0, 5, 20],
    [2, 1, 0, 1, 9],
    [0, 2, 1, 1, 12],
    [1, 0, 2, 1, 12],
    [2, 2, 2, 0, 13],
    [2, 2, 0, 2, 15],
    [2, 0, 2, 2, 17],
    [0, 2, 2, 2, 19],
    [1, 1, 1, 1, 12],
    [3, 1, 1, 1, 14],
    [1, 3, 1, 1, 16],
    [1, 1, 3, 1, 18],
    [1, 1, 1, 3, 20],
];

static INITIAL_SPELLS_ARRAY: &'static [[i8; 4]] =
    &[[2, 0, 0, 0], [-1, 1, 0, 0], [0, -1, 1, 0], [0, 0, -1, 1]];

// static ORDERS: &'static [Order] = &[Order {
//     reward: 1,
//     brewing_price: [1, 2, 0, 0],
//     action_id: 1,
//
// }];
// static LEARNABLE_SPELLS: &'static [SpellDescriptor] = &[
//     SpellDescriptor {
//         casting_price: [1, 2, -1, 0],
//         action_id: 0,
//         repeatable: true,
//     },
//     SpellDescriptor {
//         casting_price: [1, 2, -1, 0],
//         action_id: 0,
//         repeatable: true,
//     },
// ];
// static INITIAL_SPELLS: &'static [SpellDescriptor] = &[
//     SpellDescriptor {
//         casting_price: [2, 0, 0, 0],
//         action_id: 1,
//         repeatable: true,
//     },
//     SpellDescriptor {
//         casting_price: [-1, 1, 0, 0],
//         action_id: 2,
//         repeatable: true,
//     },
//     SpellDescriptor {
//         casting_price: [0, -1, 1, 0],
//         action_id: 3,
//         repeatable: true,
//     },
//     SpellDescriptor {
//         casting_price: [0, 0, -1, 1],
//         action_id: 4,
//         repeatable: true,
//     },
// ];

fn can_afford(inventory: Inventory, cost: Inventory) -> bool {
    inventory[0] > (-cost[0])
        && inventory[1] > (-cost[1])
        && inventory[2] > (-cost[2])
        && inventory[3] > (-cost[3])
}

fn cast(inventory: &mut Inventory, cost: Inventory) {
    inventory[0] += cost[0];
    inventory[1] += cost[1];
    inventory[2] += cost[2];
    inventory[3] += cost[3];
}

fn apply_move(state: &mut GameState, m: &Move) {
    match m {
        Move::Wait => {
            for x in state.spells.iter_mut() {
                x.exhausted = false;
            }
        }
        Move::Brew { action_id } => {
            let order = state
                .orders
                .iter()
                .find(|x| x.action_id == *action_id)
                .expect("Order not found");
            if !can_afford(state.inventory, order.brewing_price) {
                panic!("Cannot afford");
            }
            state.score = state.score + order.reward;
            state.potions_brewed = state.potions_brewed + 1;
            cast(&mut state.inventory, order.brewing_price);
            let next = state.next_orders.pop_front().expect("No items left");
            state.orders.push_back(next);
        }
        Move::Learn { action_id } => {
            let spell_index = state
                .learnable_spells
                .iter()
                .position(|x| x.descriptor.action_id == *action_id)
                .expect("Spell not found");
            let spell = state
                .learnable_spells
                .get(spell_index)
                .expect("Spell not found");
            let spell_cost = [-(spell_index as i8), 0, 0, 0];
            if !can_afford(state.inventory, spell_cost) {
                panic!("cannot afford learning")
            }
            cast(&mut state.inventory, spell_cost);
            let new_spell = Spell {
                descriptor: spell.descriptor,
                exhausted: false,
            };
            state.spells.push(new_spell);
            for i in 0..spell_index {
                let mut s = state
                    .learnable_spells
                    .get_mut(i)
                    .expect("index out of range");
                s.reward = s.reward + 1;
            }
            let next_learnable = state.next_learnable_spells.pop_front();
            if let Some(x) = next_learnable {
                state.learnable_spells.push(LearnableSpell {
                    descriptor: x,
                    reward: 0,
                });
            }
        }
        Move::Cast { action_id, times } => {
            let mut spell = state
                .spells
                .iter_mut()
                .find(|x| x.descriptor.action_id == *action_id)
                .expect("Spell not found");
            if !can_afford(state.inventory, spell.descriptor.casting_price) {
                panic!("Cannot afford");
            }
            if spell.exhausted {
                panic!("Spell exhausted")
            }

            for _ in 0..*times {
                cast(&mut state.inventory, spell.descriptor.casting_price);
            }

            spell.exhausted = true;
        }
    }
}

fn main() {
    let mut next_id = 0;
    let mut get_next_id = || {
        next_id += 1;
        return next_id;
    };

    let orders = ORDERS_ARRAY
        .iter()
        .map(|x| Order {
            action_id: get_next_id(),
            reward: x[4] as u32,
            brewing_price: [x[0] as i8, x[1] as i8, x[2] as i8, x[3] as i8],
        })
        .collect::<Vec<Order>>();
    let learnable_spells = SPELLS_ARRAY
        .iter()
        .map(|x| SpellDescriptor {
            action_id: get_next_id(),
            casting_price: *x,
            repeatable: x.iter().any(|x| *x < 0),
        })
        .collect::<Vec<SpellDescriptor>>();
    let initial_spells = INITIAL_SPELLS_ARRAY
        .iter()
        .map(|x| SpellDescriptor {
            action_id: get_next_id(),
            casting_price: *x,
            repeatable: x.iter().any(|x| *x < 0),
        })
        .collect::<Vec<SpellDescriptor>>();

    let mut state = make_initial_state(
        orders.as_slice(),
        learnable_spells.as_slice(),
        initial_spells.as_slice(),
    );
    while state.potions_brewed < 6 && state.moves < 100 {
        let next_move = get_move(&state);
        apply_move(&mut state, &next_move);
        state.moves = state.moves + 1;
        println!("Score: {}", state.score);
    }
}
