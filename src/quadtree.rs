use std::{collections::VecDeque, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub struct Extent {
    pub x_low: f64,
    pub x_high: f64,
    pub y_low: f64,
    pub y_high: f64,
}

pub struct QuadTree<T: Clone + Debug> {
    bot_left: Option<Box<QuadTree<T>>>,
    bot_right: Option<Box<QuadTree<T>>>,
    top_left: Option<Box<QuadTree<T>>>,
    top_right: Option<Box<QuadTree<T>>>,
    has_children: bool,
    value: Option<(VecDeque<T>, Coordinate)>,
    extent: Extent,
}

#[derive(Debug, Clone)]
pub struct Coordinate {
    x: f64,
    y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Extent {
    pub fn new(x_low: f64, y_low: f64, x_high: f64, y_high: f64) -> Self {
        Self {
            x_low,
            y_low,
            x_high,
            y_high,
        }
    }

    pub fn contains(&self, coord: &Coordinate) -> bool {
        let x = coord.x;
        let y = coord.y;
        x >= self.x_low && x <= self.x_high && y >= self.y_low && y <= self.y_high
    }

    pub fn intersects(&self, other: &Extent) -> bool {
        self.x_low <= other.x_high
            && self.x_high >= other.x_low
            && self.y_low <= other.y_high
            && self.y_high >= other.y_low
    }


    pub fn quadrant(&self, coord: &Coordinate) -> usize {
        let x = coord.x;
        let y = coord.y;
        //outside
        if !self.contains(coord) {
            panic!("Coordinate not in extent");
        }
        if x <= (self.x_low + self.x_high) / 2.0 {
            if y <= (self.y_low + self.y_high) / 2.0 {
                0 // bot left
            } else {
                2 // top left
            }
        } else if y <= (self.y_low + self.y_high) / 2.0 {
            1 // bot right
        } else {
            3 // top right
        }
    }
}

impl<T: Clone + Debug> QuadTree<T> {
    pub fn new(extent: Extent) -> Self {
        Self {
            bot_left: None,
            bot_right: None,
            top_left: None,
            top_right: None,
            value: None,
            extent,
            has_children: false,
        }
    }

    pub fn set_value(&mut self, value: T, coordinate: &Coordinate) -> bool {
        if let Some((data, coord)) = &mut self.value {
            if coord.x != coordinate.x || coord.y != coordinate.y {
                return false;
            }
            data.push_back(value);
        } else {
            let mut data = VecDeque::new();
            data.push_back(value);
            self.value = Some((data, coordinate.clone()));
        }

        self.has_children = false;
        return true;
    }

    pub fn divide(&mut self) {
        let extent = &self.extent;

        let nextent = Extent {
            x_low: extent.x_low,
            y_low: (extent.y_low + extent.y_high) / 2.0,
            x_high: (extent.x_low + extent.x_high) / 2.0,
            y_high: extent.y_high,
        };
        self.top_left = Some(Box::new(QuadTree::new(nextent)));

        let nextent = Extent {
            x_low: (extent.x_low + extent.x_high) / 2.0,
            y_low: (extent.y_low + extent.y_high) / 2.0,
            x_high: extent.x_high,
            y_high: extent.y_high,
        };
        self.top_right = Some(Box::new(QuadTree::new(nextent)));

        let nextent = Extent {
            x_low: extent.x_low,
            y_low: extent.y_low,
            x_high: (extent.x_low + extent.x_high) / 2.0,
            y_high: (extent.y_high + extent.y_low) / 2.0,
        };
        self.bot_left = Some(Box::new(QuadTree::new(nextent)));

        let nextent = Extent {
            x_low: (extent.x_low + extent.x_high) / 2.0,
            y_low: extent.y_low,
            x_high: extent.x_high,
            y_high: (extent.y_high + extent.y_low) / 2.0,
        };
        self.bot_right = Some(Box::new(QuadTree::new(nextent)));

        self.has_children = true;
    }

    pub fn insert(&mut self, coord: &Coordinate, new_value: T) -> bool {
        let mut node = self;

        //Prevent recursion for performance
        loop {
            //If there's no value but there are children, go to the children
            if node.has_children {
                let quadrant = node.extent.quadrant(coord);
                let new_node = match quadrant {
                    0 => node.bot_left.as_mut().unwrap(),
                    1 => node.bot_right.as_mut().unwrap(),
                    2 => node.top_left.as_mut().unwrap(),
                    3 => node.top_right.as_mut().unwrap(),
                    _ => panic!("Invalid quadrant"), //can't happen
                };
                node = new_node;
            } else {
                let (o_value, o_coord) = match node.value.clone() {
                    Some(value) => value,
                    None => {
                        node.set_value(new_value.clone(), coord);
                        break;
                    }
                };

                if node.set_value(new_value.clone(), coord) {
                    break;
                }

                node.divide();
                let new_quadrant = node.extent.quadrant(&o_coord);
                let new_node = match new_quadrant {
                    0 => node.bot_left.as_mut().unwrap(),
                    1 => node.bot_right.as_mut().unwrap(),
                    2 => node.top_left.as_mut().unwrap(),
                    3 => node.top_right.as_mut().unwrap(),
                    _ => panic!("Invalid quadrant"), //can't happen
                };

                new_node.value = Some((o_value.to_owned(), o_coord.to_owned()));
                node.value = None;
            }
        }

        return true;
    }

    pub fn find_bbox(&self, extent: &Extent) -> Vec<&T> {
        let mut result = Vec::new();
        let mut stack = Vec::new();
        stack.push(self);

        while let Some(node) = stack.pop() {
            if node.extent.intersects(extent) {
                if let Some((data, _)) = &node.value {
                    result.extend(data.iter());
                }

                if node.has_children {
                    if let Some(child) = &node.bot_left {
                        stack.push(child);
                    }
                    if let Some(child) = &node.bot_right {
                        stack.push(child);
                    }
                    if let Some(child) = &node.top_left {
                        stack.push(child);
                    }
                    if let Some(child) = &node.top_right {
                        stack.push(child);
                    }
                }
            }
        }

        result
    }

    pub fn print(&self) {
        println!("{:?}", self.extent);
        if let Some(child) = &self.bot_left {
            child.as_ref().print();
        }

        if let Some(child) = &self.bot_right {
            child.as_ref().print();
        }

        if let Some(child) = &self.top_left {
            child.as_ref().print();
        }

        if let Some(child) = &self.top_right {
            child.as_ref().print();
        }

        println!("{:?}", self.value);
        println!("");
    }
}
