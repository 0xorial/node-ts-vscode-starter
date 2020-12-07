use crate::solution::SearchMove::{GameMove, Initial};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::rc::Rc;

pub type Inventory = [i8; 4];

#[derive(Debug)]
pub struct SpellDescriptor {
    pub casting_price: Inventory,
    pub action_id: u32,
    pub repeatable: bool,
}

#[derive(Debug, Clone)]
pub struct Spell<'a> {
    pub descriptor: &'a SpellDescriptor,
    pub exhausted: bool,
}

#[derive(Debug, Clone)]
pub struct LearnableSpell<'a> {
    pub descriptor: &'a SpellDescriptor,
    pub reward: u32,
}

#[derive(Debug)]
pub struct Order {
    pub brewing_price: Inventory,
    pub reward: u32,
    pub action_id: u32,
}

pub struct GameState<'a> {
    pub inventory: Inventory,
    pub score: u32,

    pub spells: Vec<Spell<'a>>,

    pub learnable_spells: Vec<LearnableSpell<'a>>,
    pub next_learnable_spells: VecDeque<&'a SpellDescriptor>,

    pub orders: VecDeque<&'a Order>,
    pub next_orders: VecDeque<&'a Order>,

    pub potions_brewed: u32,
    pub next_id: u32,
    pub moves: u32,
}

#[derive(Debug, Clone)]
pub struct PlayerState<'a> {
    pub inventory: Inventory,
    pub spells: Rc<Vec<Spell<'a>>>,

    pub learnable_spells: Rc<Vec<LearnableSpell<'a>>>,
    pub orders: Rc<Vec<&'a Order>>,

    pub potions_brewed: u32,
    pub score: u32,
    pub moves: u32,
}

// impl GameState {
//     fn get_next_id(&mut self) -> u32 {
//         let r = self.next_id;
//         self.next_id = self.next_id + 1;
//         r
//     }
// }

#[derive(Copy, Clone)]
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

struct SearchNode<'a> {
    estimated_score: f64,
    last_move: SearchMove,
    state_after_move: PlayerState<'a>,
    previous: Option<Rc<SearchNode<'a>>>,
}

impl PartialEq for SearchNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        return self.estimated_score == other.estimated_score;
    }
}

impl Eq for SearchNode<'_> {}

impl Ord for SearchNode<'_> {
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
impl PartialOrd for SearchNode<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_state_score(state: &PlayerState) -> f64 {
    return state.score as f64;
}

fn make_initial_search_state(state: PlayerState) -> Rc<SearchNode> {
    Rc::new(SearchNode {
        last_move: SearchMove::Initial,
        estimated_score: get_state_score(&state),
        state_after_move: state,
        previous: None,
    })
}

fn make_brew<'a>(
    previous: &Rc<SearchNode<'a>>,
    index: usize,
    order: &Order,
) -> Option<Rc<SearchNode<'a>>> {
    let previous_state = &previous.state_after_move;
    if !can_afford(previous_state.inventory, order.brewing_price) {
        return None;
    }
    let mut new_state = previous_state.clone();
    cast(&mut new_state.inventory, order.brewing_price);
    new_state.potions_brewed += 1;
    new_state.score = new_state.score + order.reward;
    let mut new_orders = new_state.orders.as_ref().clone();
    new_orders.swap_remove(index);
    new_state.orders = Rc::new(new_orders);
    return Some(Rc::new(SearchNode {
        last_move: GameMove(Move::Brew {
            action_id: order.action_id,
        }),
        estimated_score: get_state_score(&new_state),
        state_after_move: new_state,
        previous: Some(Rc::clone(previous)),
    }));
}

fn make_cast<'a>(
    previous: &Rc<SearchNode<'a>>,
    index: usize,
    spell: &Spell,
    times: u32,
) -> Option<Rc<SearchNode<'a>>> {
    let previous_state = &previous.state_after_move;
    if spell.exhausted {
        return None;
    }
    if times > 1 && !spell.descriptor.repeatable {
        return None;
    }
    if !can_afford(previous_state.inventory, spell.descriptor.casting_price) {
        return None;
    }
    let mut new_state = previous_state.clone();
    cast(&mut new_state.inventory, spell.descriptor.casting_price);
    let mut new_spells = new_state.spells.as_ref().clone();
    let new_spell = new_spells.get_mut(index).expect("No spell");
    new_spell.exhausted = true;
    new_state.spells = Rc::new(new_spells);
    return Some(Rc::new(SearchNode {
        last_move: GameMove(Move::Cast {
            action_id: spell.descriptor.action_id,
            times,
        }),
        estimated_score: get_state_score(&new_state),
        state_after_move: new_state,
        previous: Some(Rc::clone(previous)),
    }));
}

fn make_learn<'a>(
    previous: &Rc<SearchNode<'a>>,
    index: usize,
    learnable: &LearnableSpell<'a>,
) -> Option<Rc<SearchNode<'a>>> {
    let previous_state = &previous.state_after_move;
    let learn_price = [-(index as i8), 0, 0, 0];
    if !can_afford(previous_state.inventory, learn_price) {
        return None;
    }
    let mut new_state = previous_state.clone();
    let learn_price = [((learnable.reward as i32) - (index as i32)) as i8, 0, 0, 0];
    cast(&mut new_state.inventory, learn_price);
    let mut new_spells = new_state.spells.as_ref().clone();
    new_spells.push(Spell {
        descriptor: learnable.descriptor,
        exhausted: false,
    });
    new_state.spells = Rc::new(new_spells);
    let mut new_learnable = new_state.learnable_spells.as_ref().clone();
    new_learnable.swap_remove(index);
    for i in 0..index {
        let l = new_learnable.get_mut(i).expect("Index out of range");
        l.reward += 1;
    }
    new_state.learnable_spells = Rc::new(new_learnable);
    return Some(Rc::new(SearchNode {
        last_move: GameMove(Move::Learn {
            action_id: learnable.descriptor.action_id,
        }),
        estimated_score: get_state_score(&new_state),
        state_after_move: new_state,
        previous: Some(Rc::clone(previous)),
    }));
}

fn make_wait<'a>(previous: &Rc<SearchNode<'a>>) -> Option<Rc<SearchNode<'a>>> {
    let previous_state = &previous.state_after_move;
    let mut new_state = previous_state.clone();
    let mut new_spells = new_state.spells.as_ref().clone();
    for mut s in new_spells.iter_mut() {
        s.exhausted = false;
    }
    new_state.spells = Rc::new(new_spells);
    return Some(Rc::new(SearchNode {
        last_move: GameMove(Move::Wait),
        estimated_score: get_state_score(&new_state),
        state_after_move: new_state,
        previous: Some(Rc::clone(previous)),
    }));
}

fn push<'a>(heap: &mut BinaryHeap<Rc<SearchNode<'a>>>, x: Option<Rc<SearchNode<'a>>>) {
    if let Some(x) = x {
        heap.push(x);
    }
}

fn do_search(state: PlayerState) -> Option<Rc<SearchNode>> {
    let mut heap = BinaryHeap::new();
    heap.push(make_initial_search_state(state));

    let mut steps = 0;
    while let Some(state_x) = heap.pop() {
        if steps > 1000 {
            break;
        }
        steps += 1;
        let state = &state_x;
        for (i, brew) in state.state_after_move.orders.iter().enumerate() {
            push(&mut heap, make_brew(state, i, brew));
        }
        for (i, learnable) in state.state_after_move.learnable_spells.iter().enumerate() {
            push(&mut heap, make_learn(state, i, learnable));
        }
        for (i, spell) in state.state_after_move.spells.iter().enumerate() {
            push(&mut heap, make_cast(state, i, spell, 1));
            push(&mut heap, make_cast(state, i, spell, 2));
            push(&mut heap, make_cast(state, i, spell, 3));
        }
        // push(&mut heap, make_wait(state));
    }

    let best = heap.pop();
    return best;
}

fn build_seq<'a>(node: Rc<SearchNode<'a>>) -> Vec<Rc<SearchNode<'a>>> {
    let mut result = Vec::new();
    let mut n = Some(node);
    while let Some(nn) = n {
        if let Initial = nn.last_move {
            break;
        }
        n = nn.previous.clone();
        result.push(nn);
    }
    return result;
}

fn print_seq(node: Vec<Rc<SearchNode>>) {
    for n in node.iter() {
        if let GameMove(m) = &n.last_move {
            match m {
                Move::Wait => println!("WAIT"),
                Move::Brew { .. } => println!("BREW"),
                Move::Learn { .. } => println!("LEARN"),
                Move::Cast { .. } => println!("CAST"),
            }
        } else {
            println!("INITIAL");
        }
    }
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

    return if let Some(result) = result {
        let s = build_seq(result.clone());
        print_seq(s);
        match &result.last_move {
            GameMove(m) => *m,
            SearchMove::Initial => Move::Wait,
        }
    } else {
        Move::Wait
    };
}
