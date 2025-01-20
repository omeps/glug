//!termpin terminal rationer
//!divvies up the terminal

use std::{
    fmt::Debug,
    hash::Hash,
    mem::swap,
    ops::{Add, Sub},
    sync::Arc,
};
type DivLocation = Arc<dyn Fn(usize) -> usize + Send + Sync>;
#[derive(Debug, Copy, Clone)]
pub struct Box2D<T> {
    pub x: T,
    pub y: T,
    pub length: T,
    pub height: T,
}
impl<T: PartialOrd + Debug + Add<Output = T> + Sub<Output = T> + Copy> Box2D<T> {
    fn div_hori(&self, div: &dyn Fn(T) -> T) -> Result<(Box2D<T>, Box2D<T>), String> {
        let div = div(self.height);
        if self.height < div {
            return Err(format!(
                "tried to partition a box of height {:?} by {:?} horizontally",
                self.height, div
            ));
        }
        return Ok((
            Box2D {
                height: div,
                ..*self
            },
            Box2D {
                y: self.y + div,
                height: self.height - div,
                ..*self
            },
        ));
    }
    fn div_vert(&self, div: &dyn Fn(T) -> T) -> Result<(Box2D<T>, Box2D<T>), String> {
        let div = div(self.length);
        if self.length < div {
            return Err(format!(
                "tried to partition a box of length {:?} by {:?} vertically",
                self.length, div
            ));
        }
        return Ok((
            Box2D {
                length: div,
                ..*self
            },
            Box2D {
                x: self.x + div,
                length: self.length - div,
                ..*self
            },
        ));
    }
}
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl From<termsize::Size> for Box2D<usize> {
    fn from(value: termsize::Size) -> Self {
        Self {
            x: 0,
            y: 0,
            length: value.cols as usize,
            height: value.rows as usize,
        }
    }
}
#[derive(Clone)]
pub enum DivNode<K: Eq + Hash> {
    SplitVert(DivLocation, Box<DivNode<K>>, Box<DivNode<K>>),
    SplitHori(DivLocation, Box<DivNode<K>>, Box<DivNode<K>>),
    Element(Arc<dyn Fn(Box2D<usize>, &super::gstore::GStore<'_, K>) + Send + Sync>),
    Empty,
}
impl<T: Eq + Hash> DivNode<T> {
    pub fn descend(
        &mut self,
        rect: Box2D<usize>,
        store: &super::gstore::GStore<T>,
    ) -> Result<(), String> {
        match self {
            Self::SplitVert(div, left, right) => {
                let div = rect.div_vert(&**div)?;
                let r = left.descend(div.0, store);
                right.descend(div.1, store)?;
                r?
            }
            Self::SplitHori(div, top, bottom) => {
                let div = rect.div_hori(&**div)?;
                let r = top.descend(div.0, store);
                bottom.descend(div.1, store)?;
                r?
            }
            Self::Element(disp) => disp(rect, store),
            Self::Empty => (),
        };
        Ok(())
    }
    pub fn place(&mut self, other: DivNode<T>, div: (Direction, DivLocation)) {
        use Direction::*;
        use DivNode::*;
        let mut placeholder = Empty;
        swap(self, &mut placeholder);
        *self = match div.0 {
            Up => SplitHori(div.1, Box::new(other), Box::new(placeholder)),
            Down => SplitHori(div.1, Box::new(placeholder), Box::new(other)),
            Left => SplitVert(div.1, Box::new(other), Box::new(placeholder)),
            Right => SplitVert(div.1, Box::new(placeholder), Box::new(other)),
        }
    }
}
pub mod elements {
    type UBox = super::Box2D<usize>;
    use std::fmt::Debug;
    use std::hash::Hash;

    use super::super::gstore::GStore;
    use super::super::macurses;
    use macurses::color;
    use macurses::hide_cursor;
    use macurses::set_cursor;
    pub fn draw_logs<K: Eq + Hash>(bound: UBox, store: &GStore<K>) {
        eprint!("{}{}", color!(0), hide_cursor!());
        let mut h = (bound.y..bound.y + bound.height).rev().into_iter();
        for log in store.logs() {
            eprint!("{}", color!(store.log_colors[log.1 as usize - 1]));
            for line in nice_lines(&log.0, bound.length) {
                match h.next() {
                    Some(h) => eprint!("{}{:<2$}", set_cursor!(h, bound.x), line, bound.length),
                    None => return,
                }
            }
        }
    }
    fn nice_lines(string: &str, max_len: usize) -> Vec<String> {
        let mut lines = vec![];
        string.split('\n').for_each(|line| {
            line.chars()
                .collect::<Box<[char]>>()
                .chunks(max_len)
                .for_each(|l| lines.push(String::from_iter(l)))
        });
        lines
    }
    pub fn draw_histogram<K: Eq + Hash>(bound: UBox, store: &GStore<K>) {
        eprint!("{}{}", color!(0), hide_cursor!());
        if bound.length < 6 {
            for h in bound.y..bound.y + bound.height {
                eprint!("{}{:<2$}", set_cursor!(h, bound.x), "", bound.length);
            }
            return;
        }
        for h in 0..bound.height {
            eprint!("{}", set_cursor!(h + bound.y, bound.x + 2));
            for i in 0..5 {
                eprint!(
                    "{}{} ",
                    if store.counts_total[i] >= bound.height - h {
                        color!(7)
                    } else {
                        color!(27)
                    },
                    color!(store.log_colors[i])
                );
            }
        }
    }
    pub fn horizontal_bar<K: Eq + Hash>(bound: UBox, _: &GStore<K>) {
        eprint!("{}{}", color!(0), hide_cursor!());
        eprint!("{}", set_cursor!(bound.y, bound.x));
        for _ in bound.x..bound.length {
            eprint!("=")
        }
    }
    pub fn vertical_bar<K: Eq + Hash>(bound: UBox, _: &GStore<K>) {
        eprint!("{}{}", color!(0), hide_cursor!());
        eprint!("{}", set_cursor!(bound.y, bound.x));
        for y in bound.y..bound.height {
            eprint!("{}|", set_cursor!(bound.y + y, bound.x))
        }
    }
    pub fn summary<K: Eq + Hash + Debug>(bound: UBox, store: &GStore<K>) {
        const LOG_LEVEL: [log::Level; 5] = [
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ];
        if bound.height == 0 {
            return;
        }
        eprint!(
            "{}{}{}",
            color!(0),
            hide_cursor!(),
            set_cursor!(bound.y, bound.x)
        );
        let mut line = format!(
            "{:<6}total: {},",
            "",
            store.counts_total.iter().sum::<usize>(),
        );
        if let Some((_, counts)) = &store.counts_keyed {
            for key in counts.keys() {
                line.push_str(&format!(
                    " {:?}: {},",
                    key,
                    counts.get(key).unwrap().iter().sum::<usize>()
                ))
            }
        }
        eprint!("{:<1$}", line, bound.length);
        for i in 0..5 {
            if bound.height <= i + 1 {
                return;
            }
            eprint!(
                "{}{}",
                color!(store.log_colors[i]),
                set_cursor!(bound.y + i + 1, bound.x)
            );
            let mut line = format!("{:<6}total: {},", LOG_LEVEL[i], store.counts_total[i]);
            if let Some((_, counts)) = &store.counts_keyed {
                for key in counts.keys() {
                    line.push_str(&format!(" {:?}: {},", key, counts.get(key).unwrap()[i]))
                }
            }
            eprint!("{:<1$}", line, bound.length);
        }
    }
}
