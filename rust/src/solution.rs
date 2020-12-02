use std::collections::VecDeque;
use std::collections::BinaryHeap;
use crate::make_initial_state;
use std::cmp::Ordering;
use crate::solution::Move::Brew;
use crate::solution::SearchMove::GameMove;
use std::rc::Rc;
use std::iter::FromIterator;

pub type Inventory = [i8; 4];


#[derive(Debug)]
pub struct SpellDescriptor {
    pub casting_price: Inventory,
    pub action_id: u32,
    pub repeatable: bool,
}

#[derive(Debug, Clone)]
pub struct Spell {
    pub descriptor: &'static SpellDescriptor,
    pub exhausted: bool,
}

#[derive(Debug, Clone)]
pub struct LearnableSpell {
    pub descriptor: &'static SpellDescriptor,
    pub reward: u32,
}

#[derive(Debug)]
pub struct Order {
    pub brewing_price: Inventory,
    pub reward: u32,
    pub action_id: u32,
}

pub struct GameState {
    pub inventory: Inventory,
    pub score: u32,

    pub spells: Vec<Spell>,

    pub learnable_spells: Vec<LearnableSpell>,
    pub next_learnable_spells: VecDeque<&'static SpellDescriptor>,

    pub orders: VecDeque<&'static Order>,
    pub next_orders: VecDeque<&'static Order>,

    pub potions_brewed: u32,
    pub next_id: u32,
    pub moves: u32,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub inventory: Inventory,
    pub spells: Rc<Vec<Spell>>,

    pub learnable_spells: Rc<Vec<LearnableSpell>>,
    pub orders: Rc<Vec<&'static Order>>,

    pub potions_brewed: u32,
    pub score: u32,
    pub moves: u32,
}

impl GameState {
    fn get_next_id(&mut self) -> u32 {
        let r = self.next_id;
        self.next_id = self.next_id + 1;
        r
    }
}

pub enum Move {
    Wait,
    Brew { action_id: u32 },
    Learn { action_id: u32 },
    Cast { action_id: u32, times: u32 },
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

enum SearchMove {
    GameMove(Move),
    Initial,
}

struct SearchNode {
    estimated_score: f64,
    last_move: SearchMove,
    state_after_move: PlayerState,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        return self.estimated_score == other.estimated_score;
    }
}

impl Eq for SearchNode {}


impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = other.estimated_score - self.estimated_score;
        if diff.abs() < 10e-12f64 {
            return Ordering::Equal;
        }
        if diff < 0f64 {
            return Ordering::Greater;
        }

        return Ordering::Less;
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_state_score(state: &PlayerState) -> f64 {
    return state.score as f64;
}

fn make_initial_search_state(state: PlayerState) -> SearchNode {
    SearchNode {
        last_move: SearchMove::Initial,
        estimated_score: get_state_score(&state),
        state_after_move: state,
    }
}

fn make_brew(previous: &SearchNode, index: usize, order: &Order) -> Option<SearchNode> {
    let previous_state = &previous.state_after_move;
    if !can_afford(previous_state.inventory, order.brewing_price) {
        return None;
    }
    let mut new_state = previous_state.clone();
    cast(&mut new_state.inventory, order.brewing_price);
    new_state.potions_brewed = new_state.potions_brewed + 1;
    new_state.score = new_state.score + order.reward;
    let mut new_orders = new_state.orders.as_ref().clone();
    new_orders.swap_remove(index);
    new_state.orders = Rc::new(new_orders);
    return Some(SearchNode {
        last_move: GameMove(Move::Brew { action_id: order.action_id }),
        estimated_score: get_state_score(&new_state),
        state_after_move: new_state,
    });
}

fn push(heap: &mut BinaryHeap<SearchNode>, x: Option<SearchNode>) {
    if let Some(mut x) = x {
        x.estimated_score = get_state_score(&x.state_after_move);
        heap.push(x);
    }
}

fn do_search(state: PlayerState) -> SearchNode {
    let mut heap = BinaryHeap::new();
    heap.push(make_initial_search_state(state));

    while let Some(state) = heap.pop() {
        for (i, brew) in state.state_after_move.orders.iter().enumerate() {
            push(&mut heap,make_brew(&state, i, brew));
        }
    }

    let best = heap.pop().expect("No way");
    return best;
}

pub fn get_move(state: &GameState) -> Move {
    let spells = Rc::new(state.spells.clone());
    let learnable_spells = Rc::new(state.learnable_spells.clone());
    let orders = Rc::new(Vec::from_iter(state.orders.iter().map(|x| *x)));
    let result = do_search(PlayerState {
        score: state.score,
        inventory: state.inventory,
        spells,
        learnable_spells,
        orders,
        moves: state.moves,
        potions_brewed: state.potions_brewed,
    });

    return match result.last_move {
        GameMove(m) => m,
        SearchMove::Initial => Move::Wait
    };
}
