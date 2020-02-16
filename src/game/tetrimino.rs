use termion::color::Rgb;

const BLOCK_I: [[[u8; 4]; 4]; 4] = [
    [
        [0, 0, 0, 0],
        [1, 1, 1, 1],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 1, 0],
    ],
    [
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        [1, 1, 1, 1],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
    ]
];

const BLOCK_O: [[u8; 4]; 4] = [
    [0, 1, 1, 0],
    [0, 1, 1, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

const BLOCK_T: [[[u8; 4]; 4]; 4] = [
    [
        [0, 1, 0, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ]
];

const BLOCK_S: [[[u8; 4]; 4]; 4] = [
    [
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ]
];

const BLOCK_Z: [[[u8; 4]; 4]; 4] = [
    [
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 1, 0],
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
    ]
];

const BLOCK_J: [[[u8; 4]; 4]; 4] = [
    [
        [1, 0, 0, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
    ]
];

const BLOCK_L: [[[u8; 4]; 4]; 4] = [
    [
        [0, 0, 1, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 1, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ]
];

#[derive(Copy, Clone)]
pub enum Type {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

pub struct Tetrimino {
    pub ttype: Type,
    pub state: usize,
}

impl Tetrimino {
    pub fn new(ttype: Type) -> Self {
        Tetrimino { ttype, state: 0 }
    }

    pub fn color(&self) -> Rgb {
        Self::color_of(self.ttype)
    }

    pub fn color_of(ttype: Type) -> Rgb {
        match ttype {
            Type::I => Rgb(0, 255, 255),
            Type::O => Rgb(255, 255, 0),
            Type::T => Rgb(128, 0, 128),
            Type::S => Rgb(0, 128, 0),
            Type::Z => Rgb(255, 0, 0),
            Type::J => Rgb(0, 0, 255),
            Type::L => Rgb(255, 165, 0),
        }
    }

    pub fn block(&self) -> &[[u8; 4]; 4] {
        Self::block_of(self.ttype, self.state)
    }

    pub fn block_of(ttype: Type, state: usize) -> &'static [[u8; 4]; 4] {
        match ttype {
            Type::I => BLOCK_I.get(state).unwrap(),
            Type::O => &BLOCK_O,
            Type::T => BLOCK_T.get(state).unwrap(),
            Type::S => BLOCK_S.get(state).unwrap(),
            Type::Z => BLOCK_Z.get(state).unwrap(),
            Type::J => BLOCK_J.get(state).unwrap(),
            Type::L => BLOCK_L.get(state).unwrap(),
        }
    }

    pub fn rotate_clockwise(&self) -> Tetrimino {
        Tetrimino {
            ttype: self.ttype,
            state: match self.state {
                0 => 3,
                _ => self.state - 1
            },
        }
    }

    pub fn rotate_counter_clockwise(&self) -> Tetrimino {
        Tetrimino {
            ttype: self.ttype,
            state: match self.state {
                3 => 0,
                _ => self.state + 1
            },
        }
    }
}