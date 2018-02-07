use ndarray::{Array, Array2, Ix2};
use ndarray_rand::RandomExt;
use rand::distributions::Normal;
use rand::distributions::Range;
use std::f64::consts::E;
#[prelude_import]
use std::prelude::v1::*;

/*mod mockup_linalg;

mod gradient;
use gradient::lr_gradient;*/

#[no_mangle]
pub extern "C" fn hello_world() {
    let x = 1.0_f64;
    let y = x.powf(2.3);
}

#[no_mangle]
pub extern "C" fn passing(arr: &mut [f64], len: i32) {
    arr[1] = 0.0;
}

#[no_mangle]
pub extern "C" fn scalar_add(x: &Array2<f64>, s: f64) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| i.clone() + s).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn scalar_subtract(x: &Array2<f64>, s: f64) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| i.clone() - s).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn scalar_subtracted(x: &Array2<f64>, s: f64) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| s - i.clone()).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn scalar_multiply(x: &Array2<f64>, s: f64) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| i.clone() * s).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn scalar_divide(x: &Array2<f64>, s: f64) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| i.clone() / s).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn scalar_divided(x: &Array2<f64>, s: f64) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| s / i.clone()).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn pointwise_exp(x: &Array2<f64>) -> Array2<f64> {
    let v: Vec<f64> = x.iter().map(|i| E.powf(i.clone())).collect();
    let result = Array::from_shape_vec(x.shape(), v).unwrap();
    return result.into_dimensionality::<Ix2>().unwrap();
}

#[no_mangle]
pub extern "C" fn lr_gradient(
    features: &Array2<f64>,
    labels: &Array2<f64>,
    theta: &Array2<f64>,
    lambda: f64,
) -> Array2<f64> {
    let height = features.shape()[0] as f64;
    let exponent = labels * &(features.dot(theta));
    let gradient_loss = scalar_divide(
        &(-((features.t()).dot(&(labels / &(scalar_add(&pointwise_exp(&exponent), 1.0_f64)))))),
        height,
    );
    let regularization = scalar_multiply(theta, lambda);
    let result = gradient_loss + regularization;
    return result;
}

#[no_mangle]
pub extern "C" fn dp_logistic_regression(
    features: &Array2<f64>,
    labels: &Array2<f64>,
    lambda: f64,
    learning_rate: f64,
    eps: f64,
    delta: f64,
) -> Array2<f64> {
    let n = features.shape()[0] as f64;
    let l: f64 = 1.0;
    let num_iters = 5 as usize;
    let mut theta = Array2::<f64>::zeros((features.shape()[1], 1));
    let std_dev: f64 = 4.0 * l * ((num_iters as f64) * (1.0 / delta).ln()).sqrt() / (n * eps);
    for i in 1..num_iters {
        let gradient = lr_gradient(features, labels, &theta, lambda);
        let noise = Array::random(gradient.shape(), (Normal::new(0., std_dev)));
        theta = theta - learning_rate * (gradient + noise);
    }
    return theta;
}

#[no_mangle]
pub extern "C" fn lr_predict(
    features: &Array2<f64>,
    labels: &Array2<f64>,
    theta: &Array2<f64>,
) -> f64 {
    let exponent = -features.dot(theta);
    let prediction = scalar_divided(&scalar_add(&pointwise_exp(&exponent), 1.0), 1.0);
    let errors = labels - &prediction;
    let mut sum = 0.0_f64;
    for i in errors.iter() {
        sum = sum + i;
    }
    let n = features.shape()[0] as f64;
    return sum / n;
}

#[no_mangle]
pub extern "C" fn random_dataset(
    height: usize,
    width: usize,
) -> (Array2<f64>, Array2<f64>, Array2<f64>, Array2<f64>) {
    let theta = Array::random((width, 1), (Normal::new(0., 1.)));
    let training_features = Array::random((height, width), (Normal::new(0., 1.)));
    let training_exponent = -training_features.dot(&theta);
    let training_labels = scalar_divided(&scalar_add(&pointwise_exp(&training_exponent), 1.0), 1.0);
    let testing_features = Array::random((height, width), (Normal::new(0., 1.)));
    let testing_exponent = -testing_features.dot(&theta);
    let testing_labels = scalar_divided(&scalar_add(&pointwise_exp(&testing_exponent), 1.0), 1.0);
    return (
        training_features,
        training_labels,
        testing_features,
        testing_labels,
    );
}

#[no_mangle]
pub extern "C" fn test_dplr() -> f64 {
    let (training_features, training_labels, testing_features, testing_labels) =
        random_dataset(100, 5);
    let theta = dp_logistic_regression(&training_features, &training_labels, 0.0, 0.1, 1.0, 0.01);
    let accuracy = lr_predict(&testing_features, &testing_labels, &theta);
    return accuracy;
}
