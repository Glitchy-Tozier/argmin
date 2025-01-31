// Copyright 2018-2020 argmin developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! # References:
//!
//! [0] Jorge Nocedal and Stephen J. Wright (2006). Numerical Optimization.
//! Springer. ISBN 0-387-30303-0.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::default::Default;

/// Newton's method iteratively finds the stationary points of a function f by using a second order
/// approximation of f at the current point.
///
/// [Example](https://github.com/argmin-rs/argmin/blob/master/examples/newton.rs)
///
/// # References:
///
/// [0] Jorge Nocedal and Stephen J. Wright (2006). Numerical Optimization.
/// Springer. ISBN 0-387-30303-0.
#[derive(Clone, Serialize, Deserialize)]
pub struct Newton<F> {
    /// gamma
    gamma: F,
}

impl<F: ArgminFloat> Newton<F> {
    /// Constructor
    pub fn new() -> Self {
        Newton {
            gamma: F::from_f64(1.0).unwrap(),
        }
    }

    /// set gamma
    pub fn set_gamma(mut self, gamma: F) -> Result<Self, Error> {
        if gamma <= F::from_f64(0.0).unwrap() || gamma > F::from_f64(1.0).unwrap() {
            return Err(ArgminError::InvalidParameter {
                text: "Newton: gamma must be in  (0, 1].".to_string(),
            }
            .into());
        }
        self.gamma = gamma;
        Ok(self)
    }
}

impl<F: ArgminFloat> Default for Newton<F> {
    fn default() -> Newton<F> {
        Newton::new()
    }
}

impl<O, F> Solver<O> for Newton<F>
where
    O: ArgminOp<Float = F>,
    O::Param: ArgminScaledSub<O::Param, O::Float, O::Param>,
    O::Hessian: ArgminInv<O::Hessian> + ArgminDot<O::Param, O::Param>,
    F: ArgminFloat,
{
    const NAME: &'static str = "Newton method";

    fn next_iter(
        &mut self,
        op: &mut OpWrapper<O>,
        state: &IterState<O>,
    ) -> Result<ArgminIterData<O>, Error> {
        let param = state.get_param();
        let grad = op.gradient(&param)?;
        let hessian = op.hessian(&param)?;
        let new_param = param.scaled_sub(&self.gamma, &hessian.inv()?.dot(&grad));
        Ok(ArgminIterData::new().param(new_param))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_trait_impl;
    use approx::assert_relative_eq;

    test_trait_impl!(newton_method, Newton<f64>);

    #[test]
    fn test_new() {
        let solver: Newton<f64> = Newton::new();
        assert_eq!(solver.gamma.to_ne_bytes(), 1.0f64.to_ne_bytes());
    }

    #[test]
    fn test_default() {
        let solver_new: Newton<f64> = Newton::new();
        let solver_def: Newton<f64> = Newton::default();
        assert_eq!(
            solver_new.gamma.to_ne_bytes(),
            solver_def.gamma.to_ne_bytes()
        );
    }

    #[test]
    fn test_set_gamma() {
        let new_gamma = 0.5;
        let solver: Newton<f64> = Newton::new().set_gamma(new_gamma).unwrap();
        assert_eq!(solver.gamma.to_ne_bytes(), new_gamma.to_ne_bytes());

        let new_gamma = 2.0;
        let error = Newton::new().set_gamma(new_gamma).err().unwrap();
        assert_eq!(
            error.downcast_ref::<ArgminError>().unwrap().to_string(),
            "Invalid parameter: \"Newton: gamma must be in  (0, 1].\""
        );

        let new_gamma = 0.0;
        let error = Newton::new().set_gamma(new_gamma).err().unwrap();
        assert_eq!(
            error.downcast_ref::<ArgminError>().unwrap().to_string(),
            "Invalid parameter: \"Newton: gamma must be in  (0, 1].\""
        );

        let new_gamma = -1.0;
        let error = Newton::new().set_gamma(new_gamma).err().unwrap();
        assert_eq!(
            error.downcast_ref::<ArgminError>().unwrap().to_string(),
            "Invalid parameter: \"Newton: gamma must be in  (0, 1].\""
        );
    }

    #[test]
    fn test_solver() {
        use ndarray::{Array, Array1, Array2};
        struct Problem {}

        impl ArgminOp for Problem {
            type Param = Array1<f64>;
            type Output = f64;
            type Hessian = Array2<f64>;
            type Jacobian = ();
            type Float = f64;

            fn gradient(&self, _p: &Self::Param) -> Result<Self::Param, Error> {
                Ok(Array1::from_vec(vec![1.0, 2.0]))
            }

            fn hessian(&self, _p: &Self::Param) -> Result<Self::Hessian, Error> {
                Ok(Array::from_shape_vec((2, 2), vec![1.0f64, 0.0, 0.0, 1.0])?)
            }
        }

        // Single iteration, starting from [0, 0], gamma = 1
        let problem = Problem {};
        let solver: Newton<f64> = Newton::new();
        let init_param = Array1::from_vec(vec![0.0, 0.0]);

        let param = Executor::new(problem, solver, init_param)
            .max_iters(1)
            .run()
            .unwrap()
            .state
            .best_param;
        assert_relative_eq!(param[0], -1.0, epsilon = f64::EPSILON);
        assert_relative_eq!(param[1], -2.0, epsilon = f64::EPSILON);

        // Two iterations, starting from [0, 0], gamma = 1
        let problem = Problem {};
        let solver: Newton<f64> = Newton::new();
        let init_param = Array1::from_vec(vec![0.0, 0.0]);

        let param = Executor::new(problem, solver, init_param)
            .max_iters(2)
            .run()
            .unwrap()
            .state
            .best_param;
        assert_relative_eq!(param[0], -2.0, epsilon = f64::EPSILON);
        assert_relative_eq!(param[1], -4.0, epsilon = f64::EPSILON);

        // Single iteration, starting from [0, 0], gamma = 0.5
        let problem = Problem {};
        let solver: Newton<f64> = Newton::new().set_gamma(0.5).unwrap();
        let init_param = Array1::from_vec(vec![0.0, 0.0]);

        let param = Executor::new(problem, solver, init_param)
            .max_iters(1)
            .run()
            .unwrap()
            .state
            .best_param;
        assert_relative_eq!(param[0], -0.5, epsilon = f64::EPSILON);
        assert_relative_eq!(param[1], -1.0, epsilon = f64::EPSILON);

        // Two iterations, starting from [0, 0], gamma = 0.5
        let problem = Problem {};
        let solver: Newton<f64> = Newton::new().set_gamma(0.5).unwrap();
        let init_param = Array1::from_vec(vec![0.0, 0.0]);

        let param = Executor::new(problem, solver, init_param)
            .max_iters(2)
            .run()
            .unwrap()
            .state
            .best_param;
        assert_relative_eq!(param[0], -1.0, epsilon = f64::EPSILON);
        assert_relative_eq!(param[1], -2.0, epsilon = f64::EPSILON);
    }
}
