mod solution;

use std::collections::VecDeque;
use rand_hc::Hc128Rng;
use rand::seq::SliceRandom;
use rand::{SeedableRng};
use std::iter::FromIterator;
use crate::solution::{GameState, Move, Order, SpellDescriptor, LearnableSpell, Spell, Inventory, get_move};


fn make_initial_state(all_potions: &'static [Order],
                      all_spells: &'static [SpellDescriptor]) -> GameState {
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
        spells: Vec::from_iter(INITIAL_SPELLS.iter().map(|x| {
            Spell {
                descriptor: x,
                exhausted: false,
            }
        })),
    }
}

static ORDERS: &'static [Order] = &[Order { reward: 1, brewing_price: [1, 2, 0, 0], action_id: 1 }];
static LEARNABLE_SPELLS: &'static [SpellDescriptor] = &[
    SpellDescriptor { casting_price: [1, 2, -1, 0], action_id: 0, repeatable: true },
];
static INITIAL_SPELLS: &'static [SpellDescriptor] = &[
    SpellDescriptor { casting_price: [1, 2, -1, 0], action_id: 0, repeatable: true }
];

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
            let order = state.orders.iter()
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
            let spell_index = state.learnable_spells.iter()
                .position(|x| { x.descriptor.action_id == *action_id })
                .expect("Spell not found");
            let spell = state.learnable_spells.get(spell_index)
                .expect("Spell not found");
            let spell_cost = [-(spell_index as i8), 0, 0, 0];
            if !can_afford(state.inventory, spell_cost) {
                panic!("cannot afford learning")
            }
            cast(&mut state.inventory, spell_cost);
            let new_spell = Spell {
                descriptor: spell.descriptor,
                exhausted: false
            };
            state.spells.push(new_spell);
            for i in 0..spell_index {
                let mut s = state.learnable_spells.get_mut(i)
                    .expect("index out of range");
                s.reward = s.reward + 1;
            }
            let next_learnable = state.next_learnable_spells.pop_front();
            if let Some(x) = next_learnable {
                state.learnable_spells.push(LearnableSpell {
                    descriptor: x,
                    reward: 0
                });
            }
        }
        Move::Cast { action_id, times } => {
            let mut spell = state.spells.iter_mut()
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
    let mut state = make_initial_state(ORDERS, LEARNABLE_SPELLS);
    while state.potions_brewed < 6 && state.moves < 100 {
        let next_move = get_move(&state);
        apply_move(&mut state, &next_move);
        state.moves = state.moves + 1;
        println!("Score: {}", state.score);
    }
}