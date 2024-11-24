use crate::prelude::*;
use ndarray_linalg::Inverse;
use sort_track::error::*;

type Result<T, E = KalmanError> = std::result::Result<T, E>;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct KalmanFilter {
    pub x: Array2<f64>,
    pub P: Array2<f64>,
    pub x_prior: Array2<f64>,
    pub P_prior: Array2<f64>,
    pub x_post: Array2<f64>,
    pub P_post: Array2<f64>,
    pub z: Option<Array2<f64>>,
    pub R: Array2<f64>,
    pub Q: Array2<f64>,
    pub B: Option<Array2<f64>>,
    pub F: Array2<f64>,
    pub H: Array2<f64>,
    pub y: Array2<f64>,
    pub K: Array2<f64>,
    pub S: Array2<f64>,
    pub SI: Array2<f64>,
    pub alpha_sq: f64,
}

#[allow(non_snake_case)]
impl KalmanFilter {
    pub fn new(xdim: usize, zdim: usize) -> Self {
        let x = Array2::<f64>::zeros((xdim, 1));
        let P = Array2::<f64>::eye(xdim);
        let Q = Array2::<f64>::eye(xdim);
        let F = Array2::<f64>::eye(xdim);
        let H = Array2::<f64>::zeros((zdim, xdim));
        let R = Array2::<f64>::eye(zdim);
        let alpha_sq = 1.;

        let K = Array2::<f64>::zeros((xdim, zdim));
        let y = Array2::<f64>::ones((zdim, 1));
        let S = Array2::<f64>::zeros((zdim, zdim));
        let SI = Array2::<f64>::zeros((zdim, zdim));

        let x_prior = x.clone();
        let x_post = x.clone();

        let P_prior = P.clone();
        let P_post = P.clone();

        Self {
            x,
            P,
            x_prior,
            P_prior,
            x_post,
            P_post,
            z: None,
            R,
            Q,
            B: None,
            F,
            H,
            y,
            K,
            S,
            SI,
            alpha_sq,
        }
    }

    pub fn predict(
        &mut self,
        u: Option<&Array1<f64>>,
        B: Option<&Array2<f64>>,
        F: Option<&Array2<f64>>,
        Q: Option<&Array2<f64>>,
    ) -> Result<()> {
        let B = B.or(self.B.as_ref());
        let F = F.cloned().unwrap_or(self.F.clone());
        let Q = Q.cloned().unwrap_or(self.Q.clone());

        match (B, u) {
            (Some(B), Some(u)) => self.x = &F * &self.x + B * u,
            _ => self.x = &F * &self.x,
        }

        self.P = ((&F * &self.P) * F.t()) * self.alpha_sq + Q;
        self.x_prior = self.x.clone();
        self.P_prior = self.P.clone();

        Ok(())
    }

    pub fn update(
        &mut self,
        z: &Array2<f64>,
        R: Option<&Array2<f64>>,
        H: Option<&Array2<f64>>,
    ) -> Result<()> {
        let R = R.unwrap_or(&self.R);
        let H = H.unwrap_or(&self.H);

        self.y = z - H * &self.x;

        let pht = &self.P * H.t().to_owned();
        self.S = H * &pht + R;

        self.SI = self.S.inv()?;
        self.K = pht * &self.SI;
        self.x = &self.x + &self.K * &self.y;

        let i_kh = Array2::<f64>::eye(2) - &self.K * H;
        self.P = ((&i_kh * &self.P) * i_kh.t()) + ((&self.K * R) * &self.K.t());

        self.z = Some(z.clone());
        self.x_post = self.x.clone();
        self.P_post = self.P.clone();

        Ok(())
    }
}
