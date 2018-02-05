use protobuf;
use rusty_machine::prelude::*;

use api::*;
use libcontract_common::ContractError;

/// Unpacks specified features from `tf.Example`s into a data matrix
pub fn unpack_feature_matrix(
    examples: &[Example],
    feature_names: &Vec<String>,
) -> Result<Matrix<f64>, String> {
    let vecs = examples
        .iter()
        .map(|example| {
            let mut vals = Vec::new();
            let feature_vals = &example.get_features().feature;
            for name in feature_names.iter() {
                vals.extend(match feature_vals.get(name) {
                    Some(fv) => fv.get_float_list().get_value().iter().map(|&v| v as f64),
                    None => return Err(format!("Missing feature: {}", name)),
                });
            }
            Ok(vals)
        })
        .collect::<Result<Vec<Vec<f64>>, String>>()?;

    Ok(vecs.iter().map(Vec::as_slice).collect())
}

pub fn unpack_feature_vector(
    examples: &[Example],
    feature_name: &str,
) -> Result<Vector<f64>, String> {
    let vals = examples
        .iter()
        .map(|example| {
            let val = example
                .get_features()
                .feature
                .get(feature_name)
                .map(|fv| {
                    fv.get_float_list()
                        .get_value()
                        .first()
                        .map(|&v| v as f64)
                        .ok_or("Missing feature value".to_string())
                })
                .ok_or(format!("Missing feature: {}", feature_name));
            match val {
                Ok(Ok(val)) => Ok(val),
                Err(err) | Ok(Err(err)) => Err(err),
            }
        })
        .collect::<Result<Vec<f64>, String>>()?;

    Ok(Vector::new(vals))
}

pub fn pack_proto(
    specs: Vec<(String, Matrix<f64>)>,
) -> Result<protobuf::RepeatedField<Example>, ContractError> {
    let mut lengths = specs.iter().map(|&(_, ref m)| m.rows());
    let len: usize = lengths.next().unwrap_or_default();
    if !lengths.all(|l| l == len) {
        return Err(ContractError::new(
            "Could not pack proto with matrices of different lengths",
        ));
    }

    let mut examples = Vec::with_capacity(len);
    for i in 0..len {
        let mut example = Example::new();
        {
            let mut features = example.mut_features().mut_feature();
            for &(ref name, ref vals) in specs.iter() {
                let mut feature = Feature::new();
                let mut floats = FloatList::new();
                floats.set_value(vals.row(i).iter().map(|&v| v as f32).collect());
                feature.set_float_list(floats);
                features.insert(name.clone(), feature);
            }
        }
        examples.push(example);
    }

    Ok(protobuf::RepeatedField::from_vec(examples))
}
