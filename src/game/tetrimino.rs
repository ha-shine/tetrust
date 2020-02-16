use termion::color::Rgb;

#[derive(Copy, Clone)]
pub enum TetriminoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetriminoType {
    pub fn get_color(&self) -> Rgb {
        match self {
            TetriminoType::I => Rgb(0, 255, 255),
            TetriminoType::O => Rgb(255, 255, 0),
            TetriminoType::T => Rgb(128, 0, 128),
            TetriminoType::S => Rgb(0, 128, 0),
            TetriminoType::Z => Rgb(255, 0, 0),
            TetriminoType::J => Rgb(0, 0, 255),
            TetriminoType::L => Rgb(255, 165, 0),
        }
    }

    pub fn get_block(&self, state: usize) -> &[[u8; 4]; 4] {
        match self {
            TetriminoType::I => {
                [
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
                ].get(state).unwrap()
            }
            TetriminoType::O => {
                &[
                    [0, 1, 1, 0],
                    [0, 1, 1, 0],
                    [0, 0, 0, 0],
                    [0, 0, 0, 0],
                ]
            }
            TetriminoType::T => {
                [
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
                ].get(state).unwrap()
            }
            TetriminoType::S => {
                [
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
                ].get(state).unwrap()
            }
            TetriminoType::Z => {
                [
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
                ].get(state).unwrap()
            }
            TetriminoType::J => {
                [
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
                ].get(state).unwrap()
            }
            TetriminoType::L => {
                [
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
                ].get(state).unwrap()
            }
        }
    }
}

pub struct Tetrimino {
    pub ttype: TetriminoType,
    state: usize,
}

impl Tetrimino {
    pub fn new(ttype: TetriminoType) -> Self {
        Tetrimino {ttype, state: 0 }
    }

    pub fn get_color(&self) -> Rgb {
        self.ttype.get_color()
    }

    pub fn get_block(&self) -> &[[u8; 4]; 4] {
        self.ttype.get_block(self.state)
    }

    pub fn rotate_clockwise(&mut self) {
        if self.state == 0 {
            self.state = 3
        } else {
            self.state -= 1;
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        if self.state == 3 {
            self.state = 0
        } else {
            self.state += 1
        }
    }
}