use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct DecaminxCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,

  axis_pos: RefCell<Vec<NearAxis>>,
  axis_neg: RefCell<Vec<NearAxis>>,
}

impl DecaminxCreator {
  pub fn new() -> Self {
    let min_angle = 0.7619934;
    let max_angle = 1.0945351;
    let ball_radius: f32 = 18.0;
    let split_angle = min_angle + 4.0 / ball_radius;
    let split2_angle = min_angle + 1.0 / ball_radius;

    // Wolfram alpha: u^2+v^2+w^2=1, w=2*u^2-1, w^2=u*v
    let u = 0.8539500706724498779;
    let v = 0.24613487960998085;
    let w = 0.458461446402964282;

    let axis = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: u, y: v, z: w },
      Point { x: -v, y: u, z: w },
      Point { x: -u, y: -v, z: w },
      Point { x: v, y: -u, z: w },
      Point { x: u, y: -v, z: -w },
      Point { x: v, y: u, z: -w },
      Point { x: -u, y: v, z: -w },
      Point { x: -v, y: -u, z: -w },
    ];
    let normals = axis.clone();

    Self {
      axis,
      normals,
      split_cos: split_angle.cos(),
      split2_cos: split2_angle.cos(),
      ball_radius,
      axis_pos: RefCell::new(Vec::new()),
      axis_neg: RefCell::new(Vec::new()),
    }
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
   
    let r = pos.len();
    let depth = self.ball_radius - r;
    if depth > 4.0 {
      return 0;
      for i in 0..self.axis.len() {
        let a = self.axis[i];
        if dot(pos, self.axis[i]) > 0.0 {
          let dist_to_axle = cross(pos, self.axis[i]).len();
          if dist_to_axle < 1.25 {
            return 0;
          }
        }
      }

      for v in [(7.0, 7.0, 1), (-7.0, -7.0, 2)] {
        if (pos.x - v.0).abs() + (pos.y - v.1).abs() < 2.5 {
          return 2024 + v.2;
        } else if (pos.x - v.0).abs() + (pos.y - v.1).abs() < 2.8 {
          return 0;
        }
      }

      if pos.z.abs() < 0.1 {
        return 0;
      }
      if pos.z > 0.0 {
        return 2023;
      }

      return 2024;
    }

    let mut cup = false;
    for &n in &self.normals {
      let d = dot(pos, n);
      if d > 28.5 {
        return 0;
      }
      if d > 28.0 {
        cup = true;
      }
    }

    let split_cos_in;
    let split_cos_out;

    let mut middle = false;

    if depth > 0.6 {
      split_cos_in = self.split_cos;
      split_cos_out = self.split_cos;
    } else if depth > 0.4 {
      split_cos_in = self.split_cos;
      split_cos_out = self.split_cos - (depth - 0.6) * 0.15;
    } else if depth > 0.0 {
      split_cos_in = self.split_cos;
      split_cos_out = self.split2_cos;
      middle = true;
    } else if depth > -0.2 {
      split_cos_in = self.split2_cos - (depth + 0.2) * 0.15;
      split_cos_out = self.split2_cos;
    } else {
      split_cos_in = self.split2_cos;
      split_cos_out = self.split2_cos;
    }

    let mut index: PartIndex = 0;
    let mut fail = false;
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    let match_axis = |index: &mut PartIndex,
                      fail: &mut bool,
                      axis_pos: &mut Vec<NearAxis>,
                      axis_neg: &mut Vec<NearAxis>,
                      bit: usize,
                      axis: Point|
     -> bool {
      let d = dot(pos, axis);

      let cut_factor =
        (r * 0.7 + f32::max(self.ball_radius - 2.0, f32::min(r, self.ball_radius)) * 0.3);

      if d > split_cos_out * cut_factor {
        *index += (1 << bit);
        let dist = 1.0 - (d - split_cos_out * cut_factor) * 0.8;
        if dist > 0.0 {
          axis_pos.push(NearAxis { dist, pos: axis });
        }
        return true;
      } else if d > split_cos_in * cut_factor {
        *fail = true;
        return true;
      } else if d > 0.0 {
        let dist = 1.0 - (split_cos_in * cut_factor - d) * 0.8;
        if dist > 0.0 {
          axis_neg.push(NearAxis { dist, pos: axis });
        }
      }
      return false;
    };

    for i in 0..self.axis.len() {
      if match_axis(&mut index, &mut fail, &mut axis_pos, &mut axis_neg, i, self.axis[i]) {
        if fail {
          return 0;
        }
        if !cup {
          let dist_to_axle = cross(pos, self.axis[i]).len();
          let max_dist_to_axle = if depth < -1.0 { 3.2 } else { 1.35 };
          if dist_to_axle < max_dist_to_axle {
            return 0;
          }
        }
      }
    }

    if index != 1 && index != 8 { return 0; }

    if index.count_ones() == 1 {
      if depth > 3.8 {
        return 0;
      }

      let aindex = index.ilog2() as usize;
      let a = self.axis[aindex];
      let pa = a.any_perp();
      let dist_to_axle = cross(pos, a).len();
      if r > 26.5
        || dist_to_axle < 4.25 && {
          let pr = if dot(pos, pa) > 0.2 { 22.5 } else { 24.5 };
          r > pr
        }
      {
        index += 1 << 10;
      } else if r > 26.3
        || dist_to_axle < 4.35 && {
          let pr = if dot(pos, pa) > -0.2 { 22.3 } else { 24.3 };
          r > pr
        }
      {
        return 0;
      }
    }

    let validate = |ap: &[f32], k: f32| -> bool {
      let d1 = f32::max(0.0, 1.0 - k * (1.0 - ap[0]));
      let d2 = f32::max(0.0, 1.0 - k * (1.0 - ap[1]));
      return d1 * d1 + d2 * d2 < 1.0;
    };

    if axis_pos.len() == 2 && !validate(&[axis_pos[0].dist, axis_pos[1].dist], 1.0) {
      return 0;
    }

    if axis_neg.len() == 2 && !validate(&[axis_neg[0].dist, axis_neg[1].dist], 1.0) {
      return 0;
    }

    if axis_pos.len() == 1
      && axis_neg.len() == 1
      && !middle
      && !validate(&[axis_pos[0].dist, axis_neg[0].dist], 1.0)
    {
      return 0;
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
