const pieceData = [
  [
    [1, 0, 0],
    [1, 1, 1],
    [1, 1, 1]
  ],
  [
    [0, 0, 1],
    [1, 1, 1],
    [1, 1, 1]
  ],

  [
    [0, 0, 0, 1, 1],
    [1, 1, 1, 1, 1]
  ],
  [
    [1, 1, 0, 0, 0],
    [1, 1, 1, 1, 1]
  ],
  [
    [0, 1, 1, 1],
    [1, 1, 1, 1]
  ],
  [
    [1, 1, 1, 0],
    [1, 1, 1, 1]
  ]
].map((x, i) => ({id: i + 1, shape: x}));

type PieceData = {
  data: number[][];
  width: number;
  height: number;
};

class Piece {
  constructor(public width: number, public height: number, public data: number[][], public id: number) {}

  rotate(): Piece {
    const newWidth = this.height;
    const newHeight = this.width;
    const r = makeRectangleData(newWidth, newHeight);
    for (let i = 0; i < newHeight; i++) {
      for (let j = 0; j < newWidth; j++) {
        r[i][j] = this.data[j][this.width - i - 1];
      }
    }
    return new Piece(this.height, this.width, r, this.id);
  }
  getAt(x: number, y: number): number {
    return this.data[y][x];
  }

  print() {
    for (let i = 0; i < this.height; i++) {
      let line = '';
      for (let j = 0; j < this.width; j++) {
        line = line + this.getAt(j, i).toString() + ' ';
      }
      console.log(line);
    }
    console.log('');
  }
}

class Board {
  data: number[][];
  constructor(public width: number, public height: number) {
    this.data = makeRectangleData(width, height);
  }

  canPlace(p: Piece, x: number, y: number) {
    if (x + p.width > this.width) {
      return false;
    }
    if (y + p.height > this.height) {
      return false;
    }

    for (let i = 0; i < p.height; i++) {
      for (let j = 0; j < p.width; j++) {
        const a = i + y;
        const b = j + x;
        if (p.data[i][j] === 0) {
          continue;
        }
        if (this.data[a][b] !== 0) {
          return false;
        }
      }
    }
    return true;
  }

  place(p: Piece, {x, y}: {x: number; y: number}) {
    for (let i = 0; i < p.height; i++) {
      for (let j = 0; j < p.width; j++) {
        const a = i + y;
        const b = j + x;
        if (p.data[i][j] === 0) {
          continue;
        }
        if (this.data[a][b] !== 0) {
          throw new Error();
        }
        this.data[a][b] = p.id;
      }
    }
  }

  remove(p: Piece, {x, y}: {x: number; y: number}) {
    for (let i = 0; i < p.height; i++) {
      for (let j = 0; j < p.width; j++) {
        const a = i + y;
        const b = j + x;
        if (p.data[i][j] === 0) {
          continue;
        }
        if (this.data[a][b] !== p.id) {
          throw new Error();
        }
        this.data[a][b] = 0;
      }
    }
  }

  findNextPlaceFor(p: Piece, previousPlace: null | {x: number; y: number}): 'no-place' | {x: number; y: number} {
    for (let yStep = previousPlace === null ? 0 : previousPlace.y; yStep < this.height; yStep++) {
      for (let xStep = previousPlace === null ? 0 : previousPlace.x + 1; xStep < this.width; xStep++) {
        if (this.canPlace(p, xStep, yStep)) {
          return {x: xStep, y: yStep};
        }
      }
    }
    return 'no-place';
  }

  print() {
    for (let i = 0; i < this.height; i++) {
      let line = '';
      for (let j = 0; j < this.width; j++) {
        line = line + this.data[i][j].toString() + ' ';
      }
      console.log(line);
    }
    console.log('');
  }
}

function makeRectangleData(width: number, height: number) {
  const r: number[][] = [];
  for (let i = 0; i < height; i++) {
    r.push([]);
    for (let j = 0; j < width; j++) {
      r[i].push(0);
    }
  }
  return r;
}

const pieces = pieceData.map(x => {
  const p = new Piece(x.shape[0].length, x.shape.length, x.shape, x.id);
  const p1 = p.rotate();
  const p2 = p1.rotate();
  const p3 = p2.rotate();
  return {
    rotations: [p, p1, p2, p3],
    placed: false
  };
});

let b = new Board(6, 7);

const solutions: number[][][] = [];

function processSolution(data: number[][]) {
  const height = data.length;
  const width = data[0].length;
  for (const e of solutions) {
    let same = true;
    for (let yStep = 0; yStep < height; yStep++) {
      for (let xStep = 0; xStep < width; xStep++) {
        if (e[yStep][xStep] !== data[yStep][xStep]) {
          same = false;
        }
      }
    }
    if (same) {
      return false;
    }
  }

  for (const e of solutions) {
    let same = true;
    for (let yStep = 0; yStep < height; yStep++) {
      for (let xStep = 0; xStep < width; xStep++) {
        if (e[yStep][xStep] !== data[height - yStep - 1][width - xStep - 1]) {
          same = false;
        }
      }
    }
    if (same) {
      return false;
    }
  }

  const newData = makeRectangleData(width, height);
  for (let yStep = 0; yStep < height; yStep++) {
    for (let xStep = 0; xStep < width; xStep++) {
      newData[yStep][xStep] = data[yStep][xStep];
    }
  }
  solutions.push(newData);
  return true;
}

let badTries = 0;

function go(depth: number) {
  if (depth === pieces.length + 1) {
    if (processSolution(b.data)) {
      b.print();
    }
  }

  for (let i = 0; i < pieces.length; i++) {
    if (pieces[i].placed) {
      continue;
    }

    pieces[i].placed = true;
    for (const r of pieces[i].rotations) {
      let previousPlacement: {x: number; y: number} | null = null;
      while (true) {
        const placement = b.findNextPlaceFor(r, previousPlacement);
        if (placement === 'no-place') {
          break;
        } else {
          previousPlacement = placement;
          // r.print();
          b.place(r, placement);
          go(depth + 1);
          b.remove(r, placement);
        }
      }
    }
    pieces[i].placed = false;
    badTries++;
  }
}

go(1);

console.log('total solutions ' + solutions.length);
console.log('bad tries ' + badTries);
