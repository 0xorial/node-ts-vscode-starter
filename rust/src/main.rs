use std::collections::VecDeque;
use rand_hc::Hc128Rng;
use rand::seq::SliceRandom;
use rand::{SeedableRng};
use std::iter::FromIterator;

type Inventory = [i8; 4];


struct SpellDescriptor {
    casting_price: Inventory,
    action_id: u32,
    repeatable: bool,
}

struct Spell {
    descriptor: &'static SpellDescriptor,
    exhausted: bool,
}

struct LearnableSpell {
    descriptor: &'static SpellDescriptor,
    reward: u32,
}

struct Order {
    brewing_price: Inventory,
    reward: u32,
    action_id: u32,
}

struct GameState {
    inventory: Inventory,
    score: u32,

    spells: VecDeque<Spell>,

    learnable_spells: Vec<LearnableSpell>,
    next_learnable_spells: VecDeque<&'static SpellDescriptor>,

    orders: VecDeque<&'static Order>,
    next_orders: VecDeque<&'static Order>,

    potions_brewed: u32,
    next_id: u32,
    moves: u32,
}

impl GameState {
    fn get_next_id(&mut self) -> u32 {
        let r = self.next_id;
        self.next_id = self.next_id + 1;
        r
    }
}

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
        spells: VecDeque::from_iter(INITIAL_SPELLS.iter().map(|x| {
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

enum Move {
    Wait,
    Brew { action_id: u32 },
    Learn { action_id: u32 },
    Cast { action_id: u32 },
}

fn get_move(state: &GameState) -> Move {
    return Move::Wait;
}

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
                .position(|x| { x.action_id == *action_id })
                .expect("Spell not found");
            let spell = state.learnable_spells.get(spell_index);
            let spell_cost = [-(spell_index as i8), 0, 0, 0];
            if !can_afford(state.inventory, spell_cost) {
                panic!("cannot afford learning")
            }
            cast(&mut state.inventory, spell_cost);
            let new_spell = Spell {
                descriptor: spell.descriptor,

                exhausted: false
            };
            for i in 0..spell {
                let s = state.learnable_spells.get(i).expect("");
                // s.
            }
            state.spells.push_back(new_spell);
            if state.next_learnable_spells.len() > 0 {
                let next_learnable = state.next_learnable_spells.pop_front().expect("Expected spell");
                state.learnable_spells.push_back(LearnableSpell {
                    descriptor: next_learnable,
                    reward: 0
                });
            }
        }
        Move::Cast { .. } => {}
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