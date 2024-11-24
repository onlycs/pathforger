// heartlessly stolen from pySORT (https://github.com/abewley/sort/blob/master/sort.py)
// i dont understand linear algebra. these people are smarter than me
// go star their repo

use crate::prelude::*;
use sort_track::error::*;
use sort_track::kalman::KalmanFilter;

type Result<T, E = MotionTrackError> = std::result::Result<T, E>;

#[derive(Clone, Debug, PartialEq)]
pub struct BBox {
    inner: Array2<f64>,
}

impl BBox {
    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64) -> Self {
        Self {
            inner: array![[x1, x2, y1, y2]].reversed_axes(),
        }
    }

    pub fn from_x(m: &Array2<f64>) -> Self {
        let w = (m[[2, 0]] - m[[0, 0]]).sqrt();
        let h = m[[2, 0]] / w;
        let x1 = m[[0, 0]] - w / 2.0;
        let x2 = m[[1, 0]] - h / 2.0;
        let y1 = m[[0, 0]] + w / 2.0;
        let y2 = m[[1, 0]] + h / 2.0;

        Self::new(x1, x2, y1, y2)
    }

    pub fn get(&self, i: usize) -> f64 {
        self.inner[[i, 0]]
    }

    pub fn to_z(&self) -> Array2<f64> {
        let w = self.get(2) - self.get(0);
        let h = self.get(3) - self.get(1);
        let x = self.get(0) + w / 2.0;
        let y = self.get(1) + h / 2.0;
        let s = w * h;
        let r = w / h;

        array![[x, y, s, r]].reversed_axes()
    }

    pub fn as_mat(&self) -> Array2<f64> {
        self.inner.clone()
    }

    pub fn as_mat1(&self) -> Result<Array1<f64>> {
        Ok(self.as_mat().into_shape(4)?)
    }

    pub fn from_mat1(a: Array1<f64>) -> Result<Self> {
        let a = a.into_shape((1, 4))?;
        Ok(Self { inner: a })
    }
}

#[derive(Debug)]
pub struct KalmanBoxTracker {
    filter: KalmanFilter,
    time_since_update: u32,
    history: Vec<BBox>,
    hits: u32,
    hit_streak: u32,
    age: u32,
    id: u32,
}

impl KalmanBoxTracker {
    pub fn new(id: u32, bbox: BBox) -> Self {
        let mut filter = KalmanFilter::new(7, 4);

        filter.F = array![
            [1., 0., 0., 0., 1., 0., 0.],
            [0., 1., 0., 0., 0., 1., 0.],
            [0., 0., 1., 0., 0., 0., 1.],
            [0., 0., 0., 1., 0., 0., 0.],
            [0., 0., 0., 0., 1., 0., 0.],
            [0., 0., 0., 0., 0., 1., 0.],
            [0., 0., 0., 0., 0., 0., 1.]
        ];

        filter.H = array![
            [1., 0., 0., 0., 0., 0., 0.],
            [0., 1., 0., 0., 0., 0., 0.],
            [0., 0., 1., 0., 0., 0., 0.],
            [0., 0., 0., 1., 0., 0., 0.]
        ];

        filter.R.slice_mut(s![2.., 2..]).map_inplace(|x| *x = 10.);
        filter.P.slice_mut(s![4.., 4..]).map_inplace(|x| *x = 1000.);
        filter.P.map_inplace(|x| *x = 10.);
        filter.Q.slice_mut(s![-1, -1]).map_inplace(|x| *x = 0.01);
        filter.Q.slice_mut(s![4.., 4..]).map_inplace(|x| *x = 0.01);

        filter.x.slice_mut(s![4.., 0]).assign(&bbox.to_z());

        Self {
            filter,
            time_since_update: 0,
            history: vec![],
            hits: 0,
            hit_streak: 0,
            age: 0,
            id,
        }
    }

    pub fn update(&mut self, bbox: BBox) -> Result<()> {
        self.time_since_update = 0;
        self.history = vec![];

        self.hits += 1;
        self.hit_streak += 1;

        self.filter.update(&bbox.to_z(), None, None)?;

        Ok(())
    }

    pub fn predict(&mut self) -> &BBox {
        if self.filter.x[[6, 0]] + self.filter.x[[2, 0]] <= 0.0 {
            self.filter.x[[6, 0]] = 0.0;
        }

        self.filter.predict(None, None, None, None);
        self.age += 1;

        self.time_since_update += 1;
        if self.time_since_update > 1 {
            self.hit_streak = 0;
        }

        self.history.push(BBox::from_x(&self.filter.x));
        self.history.last().unwrap() // safe to unwrap, we just pushed
    }

    pub fn state(&self) -> BBox {
        return BBox::from_x(&self.filter.x);
    }
}

pub struct Sort {
    max_age: u32,
    min_hits: u32,
    iou_threshold: f64,
    trackers: Vec<KalmanBoxTracker>,
    frame_count: u32,
    id_ctr: u32,
}

impl Sort {
    pub fn new() -> Self {
        Self {
            id_ctr: 0,
            max_age: 1,
            min_hits: 3,
            iou_threshold: 0.3,
            trackers: vec![],
            frame_count: 0,
        }
    }

    fn iou_batch(bb_test: &Array2<f64>, bb_gt: &Array2<f64>) -> Array2<f64> {
        let bb_gt = bb_gt.clone().insert_axis(Axis(0));
        let bb_test = bb_test.clone().insert_axis(Axis(1));

        let broadcast = |a: ArrayView2<f64>, b: ArrayView2<f64>| {
            let (x1, y1) = a.dim();
            let (x2, y2) = b.dim();

            // zero if either is zero, otherwise largest
            let bx = x1.max(x2) * (x1 > 0 && x2 > 0) as usize;
            let by = y1.max(y2) * (y1 > 0 && y2 > 0) as usize;

            if bx == 0 || by == 0 {
                return (Array2::zeros((bx, by)), Array2::zeros((bx, by)));
            }

            let a = a.broadcast((bx, by)).unwrap().to_owned();
            let b = b.broadcast((bx, by)).unwrap().to_owned();

            (a, b)
        };

        let max = |a: ArrayView2<f64>, b: ArrayView2<f64>| {
            let (mut a, b) = broadcast(a, b);
            azip!((a in &mut a, &b in &b) *a = a.max(b));

            a
        };

        let max_scalar = |a: Array2<f64>, b: f64| a.mapv(|x| x.max(b));

        let min = |a: ArrayView2<f64>, b: ArrayView2<f64>| {
            let (mut a, b) = broadcast(a, b);
            azip!((a in &mut a, &b in &b) *a = a.min(b));

            a
        };

        let xx1 = max(bb_test.slice(s![.., .., 0]), bb_gt.slice(s![.., .., 0]));
        let yy1 = max(bb_test.slice(s![.., .., 1]), bb_gt.slice(s![.., .., 1]));
        let xx2 = min(bb_test.slice(s![.., .., 2]), bb_gt.slice(s![.., .., 2]));
        let yy2 = min(bb_test.slice(s![.., .., 3]), bb_gt.slice(s![.., .., 3]));

        let w = max_scalar(xx2 - xx1, 0.0);
        let h = max_scalar(yy2 - yy1, 0.0);
        let wh = w * h;

        #[rustfmt::skip]
        let o = &wh / (
            (
                (bb_test.slice(s![.., .., 2]).to_owned() - bb_test.slice(s![.., .., 0]))
                * (bb_test.slice(s![.., .., 3]).to_owned() - bb_test.slice(s![.., .., 1]))
            ) + (
                (bb_gt.slice(s![.., .., 2]).to_owned() - bb_gt.slice(s![.., .., 0]))
                * (bb_gt.slice(s![.., .., 3]).to_owned() - bb_gt.slice(s![.., .., 1]))
            ) - &wh
        );

        o
    }

    fn stack_nonzero(mat: Array2<u64>) -> Array2<usize> {
        let coords = mat.indexed_iter().filter(|(_, &x)| x != 0).map(|(c, _)| c);
        let xs = coords.clone().map(|(x, _)| x);
        let ys = coords.map(|(_, y)| y);

        let matx = Array1::from_iter(xs);
        let maty = Array1::from_iter(ys);

        stack![Axis(1), matx, maty]
    }

    fn linear_assignment(cost: Array2<f64>) -> Result<Array2<usize>> {
        let (x, y) = lapjv::lapjv(&cost)?;
        let iter = x.iter().flat_map(|i| [y[*i], *i]);

        Ok(Array2::from_shape_vec((x.len(), 2), iter.collect())?)
    }

    fn associate_dets(
        dets: &Array2<f64>,
        tracks: &Array2<f64>,
        iou_theshold: f64,
    ) -> Result<(Array2<usize>, Array2<usize>, Array2<usize>)> {
        if tracks.is_empty() {
            return Ok((
                Array2::zeros((0, 2)),
                Array2::from_shape_vec((1, dets.len()), (0..dets.len()).collect())?,
                Array2::zeros((0, 5)),
            ));
        }

        let iou_matrix = Self::iou_batch(&dets, &tracks);

        let matched_indecies;
        if *min(iou_matrix.shape()).unwrap() > 0 {
            let a = iou_matrix.map(|x| u64::from(*x > iou_theshold));

            if max(a.sum_axis(Axis(1))).unwrap() == 1 && max(a.sum_axis(Axis(0))).unwrap() == 1 {
                matched_indecies = Self::stack_nonzero(a);
            } else {
                matched_indecies = Self::linear_assignment(-&iou_matrix)?;
            }
        } else {
            matched_indecies = Array2::zeros((0, 2));
        }

        let mut unmatched_dets = vec![];
        for (d, _) in dets.rows().into_iter().enumerate() {
            if !matched_indecies.column(0).iter().any(|x| *x == d) {
                unmatched_dets.push(d);
            }
        }

        let mut unmatched_tracks = vec![];
        for (t, _) in tracks.rows().into_iter().enumerate() {
            if !matched_indecies.column(1).iter().any(|x| *x == t) {
                unmatched_tracks.push(t);
            }
        }

        let mut matches = vec![];
        for m in matched_indecies.rows() {
            if iou_matrix[[m[0], m[1]]] < iou_theshold {
                unmatched_dets.push(m[0]);
                unmatched_tracks.push(m[1]);
            } else {
                matches.push(m.into_shape((1, 2))?);
            }
        }

        let matchesmat = match matches.len() {
            0 => Array2::zeros((0, 2)),
            _ => concatenate(Axis(0), matches.as_slice())?,
        };

        return Ok((
            matchesmat,
            Array2::from_shape_vec((1, unmatched_dets.len()), unmatched_dets)?,
            Array2::from_shape_vec((1, unmatched_tracks.len()), unmatched_tracks)?,
        ));
    }

    /// one row in `dets`: (x1,x2,y1,y2,score) == (...bbox.as_mat(), confidence)
    /// for each object in frame
    pub fn update(&mut self, dets: Array2<f64>) -> Result<Array2<f64>> {
        self.frame_count += 1;

        let mut tracks = Array2::<f64>::zeros((self.trackers.len(), 5));
        let mut to_delete = vec![];
        let mut ret = vec![];

        for i in 0..self.trackers.len() {
            let bbox = self.trackers[i].predict().as_mat1()?;
            tracks.row_mut(i).assign(&bbox.clone().into_shape((1, 5))?);

            if bbox.iter().copied().any(f64::is_nan) {
                to_delete.push(i);
            }
        }

        for i in to_delete.iter().rev() {
            self.trackers.remove(*i);
            tracks.remove_index(Axis(0), *i);
        }

        let (matched, unmatched_dets, _) =
            Self::associate_dets(&dets, &tracks, self.iou_threshold)?;

        for m in matched.rows() {
            self.trackers[m[1]].update(BBox::from_mat1(dets.row(m[0]).to_owned())?);
        }

        for i in unmatched_dets {
            let id = self.id_ctr;
            let track = KalmanBoxTracker::new(id, BBox::from_mat1(dets.row(i).to_owned())?);
            self.trackers.push(track);
            self.id_ctr += 1;
        }

        for i in (0..self.trackers.len()).rev() {
            let trk = &self.trackers[i];
            let d = trk.state();

            if trk.time_since_update < 1
                && (trk.hit_streak >= self.min_hits || self.frame_count <= self.min_hits)
            {
                let mat = concatenate![Axis(0), d.as_mat1()?, [trk.id as f64]];
                let cols = mat.len();

                ret.push(mat.into_shape((1, cols))?);
            }

            if trk.time_since_update > self.max_age {
                self.trackers.remove(i);
            }
        }

        if ret.len() > 0 {
            Ok(Array2::from_shape_vec(
                (ret.len(), 5),
                ret.into_iter().flatten().collect(),
            )?)
        } else {
            Ok(Array2::zeros((0, 5)))
        }
    }
}
