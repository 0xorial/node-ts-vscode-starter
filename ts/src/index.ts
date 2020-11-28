let turnsLeft = 101;

const fs = require('fs');
const text: string = fs.readFileSync('./input.txt', 'utf-8');
const lines = text.split('\r\n');

let lineI = 0;
function readLine() {
  return lines[lineI++];
}

const turn = parseInt(readLine());
turnsLeft = 101 - turn;

const readline = () => readLine();

// declare var require: any;
const p: {now: () => number} = require('perf_hooks').performance;

class PriorityQueue<T> {
  comparator: (a: T, b: T) => number;
  private _memory: Array<T>;
  length = 0;

  constructor(options: {comparator: (a: T, b: T) => number; size: number}) {
    this.comparator = options.comparator;
    let shift = 0;
    while (1 << shift < options.size) {
      shift += 1;
    }
    if (1 << shift !== options.size) {
      throw 'size must be a power of two';
    }
    this._memory = new Array(options.size);
  }

  queue(value: T) {
    if (this.length === this._memory.length) throw new Error('queue overflow');
    this._memory[this.length] = value;
    this.length += 1;
    this._bubbleUp(value);
  }

  dequeue(): T {
    const r = this._memory[0];
    this.length--;
    const last = this._memory[this.length];
    this._memory[0] = last;
    this._bubbleDown(last);
    return r;
  }

  clear() {
    this.length = 0;
    this._memory.length = 0;
  }

  _bubbleUp(value: T) {
    const compare = this.comparator;
    let index = this.length - 1;
    while (index > 0) {
      const parentIndex = (index - 1) >> 1;
      const parentValue = this._memory[parentIndex];
      if (compare(parentValue, value) >= 0) {
        break;
      }
      this._memory[parentIndex] = value;
      this._memory[index] = parentValue;
      index = parentIndex;
    }
  }

  _bubbleDown(value: T) {
    const compare = this.comparator;
    let index = 0;
    while (index < this.length) {
      const childIndex1 = (index << 1) + 1;
      const childIndex2 = childIndex1 + 1;
      if (childIndex2 < this.length) {
        const child1 = this._memory[childIndex1];
        const child2 = this._memory[childIndex2];
        const childCompare = compare(child1, child2);
        const smallerChild = childCompare >= 0 ? child1 : child2;
        const parentCompare = compare(value, smallerChild);
        if (parentCompare > 0) break;

        if (childCompare >= 0) {
          this._memory[childIndex1] = value;
          this._memory[index] = child1;
          index = childIndex1;
        } else {
          this._memory[childIndex2] = value;
          this._memory[index] = child2;
          index = childIndex2;
        }
      } else if (childIndex1 < this.length) {
        const child1 = this._memory[childIndex1];
        const parentCompare = compare(value, child1);
        if (parentCompare > 0) break;
        this._memory[childIndex1] = value;
        this._memory[index] = child1;
        index = childIndex1;
      } else break;
    }
  }
}

// const h = new PriorityQueue<number>({size: 32, comparator: (a, b) => b - a});

// h.queue(3);
// h.queue(5);
// h.queue(4);
// h.queue(2);
// h.queue(10);
// h.dequeue();
// h.dequeue();
// h.dequeue();
// h.dequeue();
// h.dequeue();

type WithPrice = readonly [number, number, number, number];
type Inventory = WithPrice;
type Recipe = {
  readonly price: number;
  readonly delta: readonly [number, number, number, number];
  readonly deltaSize: number;
  readonly actionId: number;
};
type SpellDescriptor = {
  castingPrice: WithPrice;
  totalInOut: number;
  totalConsumed: number;
  totalProduced: number;
  actionId: number;
  repeatable: boolean;
};

type LearnableSpell = {
  descriptor: SpellDescriptor;
  reward: number;
  position: number;
};

type Spell = {descriptor: SpellDescriptor; exhausted: boolean};

type StepAction = (
  | {type: 'none'}
  | {type: 'wait'}
  | {type: 'cast'; id: number; times: number}
  | {type: 'brew'; recipe: Recipe}
  | {type: 'learn'; id: number}
) & {note?: string};

type Step = {
  previous: Step | null;

  inventory: WithPrice;
  inventorySize: number;

  spells: Spell[];
  learnable: LearnableSpell[];
  length: number;
  action: StepAction;
  recipes: readonly Recipe[];
  potions: readonly number[];
  distance: number;
  gameScore: number;
};

let counters: any = {};

function wrap<T extends CallableFunction>(c: T): T {
  return c;
  const cc: any = c;
  const name = c.name;
  counters[name] = 0;
  const w: any = function wrapper() {
    counters[name]++;
    return cc.apply(undefined, arguments);
  };
  return w;
}

// function printCounters() {
//   for (const k of Object.keys(counters)) {
//     console.error(k + ': ' + counters[k]);
//     counters[k] = 0;
//   }
// }

function makeInitial(inventory: WithPrice, spells: Spell[], learnable: LearnableSpell[], recipes: Recipe[]): Step {
  return {
    previous: null,
    inventory,
    inventorySize: sum(inventory),
    spells,
    learnable,
    length: 0,
    action: {type: 'none'},
    recipes,
    potions: [],
    distance: 0,
    gameScore: 0,
  };
}

const clone = wrap(function clone(step: Step, action: StepAction): Step {
  return {
    previous: step,
    inventory: step.inventory,
    inventorySize: step.inventorySize,
    spells: step.spells,
    learnable: step.learnable,
    action: action,
    length: step.length + 1,
    recipes: step.recipes,
    potions: step.potions,
    distance: 0,
    gameScore: step.gameScore,
  };
});

const sum = wrap(function sum(i: readonly [number, number, number, number]) {
  return i[0] + i[1] + i[2] + i[3];
});

const sumAny = wrap(function sumAny(i: readonly number[]) {
  let r = 0;
  for (const n of i) r += n;
  return r;
});

const sortBy = wrap(function sortBy<T>(array: T[], selector: (i: T) => number) {
  return array.sort((a, b) => {
    const aN = selector(a);
    const bN = selector(b);
    return aN > bN ? -1 : aN === bN ? 0 : 1;
  });
});

const canAfford = wrap(function canAfford(i: WithPrice, totalI: number, r: WithPrice, totalR: number) {
  return totalI >= totalR && i[0] >= -r[0] && i[1] >= -r[1] && i[2] >= -r[2] && i[3] >= -r[3];
});

const canAffordTimes = wrap(function canAffordTimes(i: WithPrice, totalI: number, s: SpellDescriptor, times: number) {
  if (totalI < s.totalConsumed * times) return false;
  const r = s.castingPrice;
  return i[0] >= -r[0] * times && i[1] >= -r[1] * times && i[2] >= -r[2] * times && i[3] >= -r[3] * times;
});

const cast = wrap(function cast(i: Inventory, r: WithPrice): [number, number, number, number] {
  return [i[0] + r[0], i[1] + r[1], i[2] + r[2], i[3] + r[3]];
});

const castTimes = wrap(function castTimes(i: Inventory, r: WithPrice, times: number): WithPrice {
  return [i[0] + r[0] * times, i[1] + r[1] * times, i[2] + r[2] * times, i[3] + r[3] * times];
});

let weights = [1, 1, 1, 1];

const makeWait = wrap(function makeWait(s: Step): Step {
  const result = clone(s, {type: 'wait'});
  result.spells = s.spells.map((x) => ({descriptor: x.descriptor, exhausted: false}));
  return result;
});

const makeBrew = wrap(function makeBrew(s: Step, recipeIndex: number): Step | null {
  const r = s.recipes[recipeIndex];
  if (!canAfford(s.inventory, s.inventorySize, r.delta, r.deltaSize)) {
    return null;
  }

  const result = clone(s, {type: 'brew', recipe: r});
  result.inventory = cast(s.inventory, r.delta);
  result.inventorySize += r.deltaSize;
  result.recipes = result.recipes.filter((_, i) => i !== recipeIndex);
  result.potions = [...result.potions, r.actionId];
  result.gameScore += r.price;
  return result;
});

const makeCast = wrap(function makeCast(s: Step, spellIndex: number, times: number): Step | null {
  const r = s.spells[spellIndex];
  if (r.exhausted) return null;
  if (!canAffordTimes(s.inventory, s.inventorySize, r.descriptor, times)) return null;
  if (s.inventorySize + r.descriptor.totalInOut > 10) return null;

  const result = clone(s, {type: 'cast', id: r.descriptor.actionId, times});
  result.inventory = castTimes(s.inventory, r.descriptor.castingPrice, times);
  result.inventorySize += r.descriptor.totalInOut * times;
  result.spells = [...s.spells];
  result.spells[spellIndex] = {exhausted: true, descriptor: r.descriptor};
  return result;
});

const makeLearn = wrap(function makeLearn(s: Step, spellIndex: number) {
  const r = s.learnable[spellIndex];
  const price = [-r.position, 0, 0, 0] as const;
  if (!canAfford(s.inventory, s.inventorySize, price, -r.position)) {
    return null;
  }

  const deltaSize = r.reward - r.position;
  const result = clone(s, {type: 'learn', id: r.descriptor.actionId});
  const resultInventory = cast(result.inventory, [deltaSize, 0, 0, 0]);
  let resultSize = result.inventorySize + deltaSize;
  const overflow = resultSize - 10;
  if (overflow > 0) {
    resultInventory[0] -= overflow;
    resultSize = 10;
  }
  result.spells = [...s.spells.map((x) => ({...x})), {descriptor: r.descriptor, exhausted: false}];
  return result;
});

function makeSeq(step: Step): Step[] {
  let r = step;
  let rr: Step[] = [];
  while (r.action.type !== 'none' && r.previous) {
    rr.push(r);
    r = r.previous;
  }
  rr = rr.reverse();
  return rr;
}

const sequenceValue = wrap(function sequenceValue(lastStep: Step, r: Recipe) {
  return (r.price / lastStep.length) * 10 + lastStep.inventorySize;
});

function printSteps(ss: Step[]) {
  // console.error('start receipt-----------------');
  // // console.error(ss)
  for (const s of ss) {
    // console.error({
    //     inventory: s.inventory,
    //     spells: s.spells,
    //     length: s.length,
    //     action: s.action,
    //     recipes: s.recipes
    // });
    if (s.action.type === 'wait') console.error(s.distance + ` REST ${s.inventory}`);
    else if (s.action.type === 'brew')
      console.error(s.distance + ` BREW ${s.action.recipe.actionId} ${s.inventory} ${s.potions}`);
    else if (s.action.type === 'cast')
      console.error(s.distance + ` CAST ${s.action.id} ${s.inventory} ${s.action.times}`);
    else if (s.action.type === 'learn') console.error(s.distance + ` LEARN ${s.action.id} ${s.inventory} `);
    else console.error((s.action as any).type);
  }
  // const last = ss[ss.length - 1];
  // if (last.action.type === 'brew') {
  //     console.error('value: ' + sequenceValue(last, last.action.recipe));
  // }
  // console.error('end rec-----------------');
}

const binaryInsert = wrap(function binaryInsert<T>(
  value: T,
  array: T[],
  s: (i: T) => number,
  startVal?: number,
  endVal?: number
) {
  var length = array.length;
  var start = typeof startVal != 'undefined' ? startVal : 0;
  var end = typeof endVal != 'undefined' ? endVal : length - 1; //!! endVal could be 0 don't use || syntax
  var m = start + Math.floor((end - start) / 2);

  var w = s(value);
  if (length == 0) {
    array.push(value);
    return;
  }

  if (w > s(array[end])) {
    array.splice(end + 1, 0, value);
    return;
  }

  if (w <= s(array[start])) {
    //!!
    array.splice(start, 0, value);
    return;
  }

  if (start >= end) {
    return;
  }

  if (w <= s(array[m])) {
    binaryInsert(value, array, s, start, m - 1);
    return;
  }

  if (w > s(array[m])) {
    binaryInsert(value, array, s, m + 1, end);
    return;
  }
});

let ppp = false;
const csvLines: string[] = [];

let indent = 0;
function timeStart(message: string, printC = false) {
  const start = p.now();
  const useIndent = ' '.repeat(indent);
  const useIndentLen = indent;
  indent++;
  console.error(useIndent + 'started: ' + message);
  return {
    startTime: start,
    timeEnd: () => {
      const end = p.now();
      indent = useIndentLen;
      const timeMs = Math.round((end - start) * 100) / 100;

      // if (printC) {
      //   if (!ppp) {
      //     let r = 'time,' + Object.keys(counters).join(',');
      //     csvLines.push(r);
      //     ppp = true;
      //   }
      //   let m =
      //     timeMs.toString() +
      //     ',' +
      //     Object.keys(counters)
      //       .map((x) => counters[x])
      //       .join(',');
      //   csvLines.push(m);
      //   for (const k of Object.keys(counters)) {
      //     counters[k] = 0;
      //   }
      // }

      const time = '---' + timeMs.toString() + 'ms--- ';
      console.error(useIndent + 'ended: in ' + time + message);
    },
  };
}

function distance(i: Inventory, r: WithPrice): number {
  const d0 = Math.max(-r[0] - i[0], 0);
  const d1 = Math.max(-r[1] - i[1], 0);
  const d2 = Math.max(-r[2] - i[2], 0);
  const d3 = Math.max(-r[3] - i[3], 0);
  return d0 + d1 + d2 + d3;
}

function stepScore(s: Step) {
  let score = 100 * s.gameScore;
  for (const p of s.recipes) {
    const potionDistanceScore = 6 - distance(s.inventory, p.delta);
    score += potionDistanceScore * p.price;
  }
  score += s.inventory[3] * 3;
  score += s.inventory[2] * 2;
  const achievedAt = 100 - turnsLeft + s.length;
  score += 2 * s.spells.length;
  score /= achievedAt;
  score += 50 * s.potions.length * Math.pow(0.8, achievedAt);
  score += 10 * s.gameScore * Math.pow(0.8, achievedAt);
  return score;
}

const maxSteps = 10000000;
const queueSize = 1048576 * 1024;
const maxTime = 35;

function find(initial: Step): Step | null {
  const t = timeStart(`looking for move`, true);
  let steps = 0;
  let bestSolution: Step | null = null;
  try {
    let q = new PriorityQueue<Step>({
      size: queueSize,
      comparator: (a, b) => {
        return a.distance - b.distance;
      },
    });
    q.queue(initial);

    function tryAdd(s: Step | null) {
      if (s) {
        s.distance = stepScore(s);
        q.queue(s);
      }
    }
    while (q.length > 0) {
      steps++;
      const step = q.dequeue();
      bestSolution = step;
      if (steps % 50 === 0) {
        if (p.now() - t.startTime > maxTime) {
          console.error('hurry! ' + steps);
          break;
        }
      }
      if (steps === maxSteps) {
        break;
      }
      for (let i = 0; i < step.recipes.length; i++) tryAdd(makeBrew(step, i));
      if (step.length > turnsLeft) continue;
      if (step.action.type !== 'wait') tryAdd(makeWait(step));
      if (step.length < 3) for (let i = 0; i < step.learnable.length; i++) tryAdd(makeLearn(step, i));
      for (let i = 0; i < step.spells.length; i++) {
        const spell = step.spells[i];
        if (spell.descriptor.repeatable) {
          tryAdd(makeCast(step, i, 1));
          tryAdd(makeCast(step, i, 2));
          tryAdd(makeCast(step, i, 3));
          tryAdd(makeCast(step, i, 4));
        } else {
          tryAdd(makeCast(step, i, 1));
        }
      }
    }

    return bestSolution;
  } catch (e) {
    console.error(e);
    return bestSolution;
  } finally {
    t.timeEnd();
  }
}

function exportPrint(a: any) {
  // console.error(a);
}

function max(items: number[]) {
  let m = items[0];
  for (let i = 1; i < items.length; i++) {
    if (items[i] > m) m = items[i];
  }
  return m;
}

function findLearnSpellGold(learnable: LearnableSpell[]): LearnableSpell | null {
  const gold = learnable.find((x) => x.descriptor.totalConsumed === 0);
  if (gold) return gold;
  return null;
}

type StepInput = {
  inventory: WithPrice;
  spells: Spell[];
  orders: Recipe[];
  learnable: LearnableSpell[];
};

function readInput(): StepInput {
  const line = readline();
  exportPrint(line);
  const actionCount: number = parseInt(line); // the number of spells and recipes in play
  const orders: Recipe[] = [];
  const spells: Spell[] = [];
  const learnable: LearnableSpell[] = [];
  for (let i = 0; i < actionCount; i++) {
    const line2 = readline();
    exportPrint(line2);
    var inputs: string[] = line2.split(' ');
    const actionId: number = parseInt(inputs[0]); // the unique ID of this spell or recipe
    const actionType: string = inputs[1]; // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
    const delta0: number = parseInt(inputs[2]); // tier-0 ingredient change
    const delta1: number = parseInt(inputs[3]); // tier-1 ingredient change
    const delta2: number = parseInt(inputs[4]); // tier-2 ingredient change
    const delta3: number = parseInt(inputs[5]); // tier-3 ingredient change
    const price: number = parseInt(inputs[6]); // the price in rupees if this is a potion
    const tomeIndex: number = parseInt(inputs[7]); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax; For brews, this is the value of the current urgency bonus
    const taxCount: number = parseInt(inputs[8]); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell; For brews, this is how many times you can still gain an urgency bonus
    const castable: boolean = inputs[9] !== '0'; // in the first league: always 0; later: 1 if this is a castable player spell
    const repeatable: boolean = inputs[10] !== '0'; // for the first two leagues: always 0; later: 1 if this is a repeatable player spell
    const deltaArray = [delta0, delta1, delta2, delta3] as const;
    const spellDescriptor = {
      actionId,
      castingPrice: deltaArray,
      repeatable,
      totalConsumed: -sumAny(deltaArray.filter((x) => x < 0)),
      totalProduced: sumAny(deltaArray.filter((x) => x > 0)),
      totalInOut: sum(deltaArray),
    };
    if (actionType === 'BREW') orders.push({actionId, delta: deltaArray, deltaSize: sum(deltaArray), price});
    if (actionType === 'CAST')
      spells.push({
        descriptor: spellDescriptor,
        exhausted: !castable,
      });
    if (actionType === 'LEARN')
      learnable.push({
        descriptor: spellDescriptor,
        position: tomeIndex,
        reward: taxCount,
      });
  }
  const inventories = [];
  for (let i = 0; i < 2; i++) {
    const line3 = readline();
    exportPrint(line3);
    var inputs: string[] = line3.split(' ');
    const inv0: number = parseInt(inputs[0]); // tier-0 ingredients in inventory
    const inv1: number = parseInt(inputs[1]);
    const inv2: number = parseInt(inputs[2]);
    const inv3: number = parseInt(inputs[3]);
    const score: number = parseInt(inputs[4]); // amount of rupees
    inventories.push([inv0, inv1, inv2, inv3] as const);
  }

  return {
    inventory: inventories[0],
    spells,
    orders,
    learnable,
  };
}

function getNextStep(input: StepInput): StepAction {
  const {inventory, learnable, orders, spells} = input;
  // const t = timeStart('finding gold spell');
  // const toLearn = findLearnSpellGold(learnable);
  // t.timeEnd();
  // const t2 = timeStart('finding way to learn gold spell');
  // try {
  //   if (toLearn != null) {
  //     const pseudoRecipe = {
  //       actionId: -1,
  //       delta: [-toLearn.position, 0, 0, 0],
  //       deltaSize: -toLearn.position,
  //       price: 10,
  //     } as const;
  //     const learnWay = find(inventory, pseudoRecipe, spells, learnable, [...orders, pseudoRecipe], 5);
  //     if (learnWay) {
  //       const learnPath = makeSeq(learnWay);
  //       if (learnPath.length < 3) {
  //         if (learnPath.length === 1) return {type: 'learn', id: toLearn.descriptor.actionId};
  //         else return learnPath[0].action;
  //       }
  //     }
  //   }
  // } finally {
  //   t2.timeEnd();
  // }

  const t3 = timeStart('finding spells brewing ways');
  try {
    const initial = makeInitial(inventory, spells, learnable, orders);
    const instruction = find(initial);
    if (!instruction) {
      return {type: 'wait'};
    }
    const seq = makeSeq(instruction);
    printSteps(seq);
    return seq[0].action;
  } finally {
    t3.timeEnd();
  }
}
try {
  // game loop
  while (true) {
    turnsLeft--;

    const t = timeStart('reading input');

    const input = readInput();
    t.timeEnd();

    const t2 = timeStart('getting next step');
    const decision = getNextStep(input);
    t2.timeEnd();
    if (decision.type === 'none') {
      console.log('WAIT' + formatNote(decision.note));
      console.error('no action???');
    } else if (decision.type === 'wait') console.log('REST' + formatNote(decision.note));
    else if (decision.type === 'learn') console.log(`LEARN ${decision.id}` + formatNote(decision.note));
    else if (decision.type === 'brew') console.log(`BREW ${decision.recipe.actionId}` + formatNote(decision.note));
    else if (decision.type === 'cast') {
      if (decision.times === 1) console.log(`CAST ${decision.id}` + formatNote(decision.note));
      else console.log(`CAST ${decision.id} ${decision.times}` + formatNote(decision.note));
    }
  }
} finally {
  // const fs = require('fs');
  // fs.writeFileSync('./data.csv', csvLines.join('\r\n'));
}

function formatNote(n: string | undefined) {
  return n ? ' ' + n : '';
}
