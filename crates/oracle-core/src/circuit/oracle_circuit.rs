//! R1CS circuit proving honest aggregation over [`crate::MAX_SOURCES`] fixed slots.

use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, SynthesisError,
};

use crate::MAX_SOURCES;

/// Bits used to range-check the weighted-majority margin witness.
pub const MAJORITY_MARGIN_BITS: usize = 25;

/// Witness and public inputs for one oracle resolution proof.
#[derive(Clone)]
pub struct OracleCircuit {
    /// Per-source outcome witness: 1 = Yes, 0 = No (Unknown treated as 0 weight).
    pub source_outcomes: Vec<Option<Fr>>,
    /// Per-source confidence weight in the scalar field.
    pub source_weights: Vec<Option<Fr>>,
    /// 1 if source survived outlier removal, else 0.
    pub source_included: Vec<Option<Fr>>,
    /// Binary decomposition of the majority margin (range-checked).
    pub majority_margin_bits: Vec<Option<Fr>>,
    /// Public: 1 = Yes, 0 = No.
    pub final_outcome: Option<Fr>,
    /// Public: count of included sources.
    pub source_count: Option<Fr>,
}

impl Default for OracleCircuit {
    fn default() -> Self {
        Self {
            source_outcomes: vec![None; MAX_SOURCES],
            source_weights: vec![None; MAX_SOURCES],
            source_included: vec![None; MAX_SOURCES],
            majority_margin_bits: vec![None; MAJORITY_MARGIN_BITS],
            final_outcome: None,
            source_count: None,
        }
    }
}

fn pad_to_max_sources(values: &[Option<Fr>]) -> Vec<Option<Fr>> {
    let mut padded = values.to_vec();
    padded.resize(MAX_SOURCES, Some(Fr::from(0u64)));
    padded.truncate(MAX_SOURCES);
    padded
}

fn pad_margin_bits(values: &[Option<Fr>]) -> Vec<Option<Fr>> {
    let mut padded = values.to_vec();
    padded.resize(MAJORITY_MARGIN_BITS, Some(Fr::from(0u64)));
    padded.truncate(MAJORITY_MARGIN_BITS);
    padded
}

impl ConstraintSynthesizer<Fr> for OracleCircuit {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<Fr>,
    ) -> Result<(), SynthesisError> {
        let source_outcomes = pad_to_max_sources(&self.source_outcomes);
        let source_weights = pad_to_max_sources(&self.source_weights);
        let source_included = pad_to_max_sources(&self.source_included);
        let majority_margin_bits = pad_margin_bits(&self.majority_margin_bits);

        let outcomes: Vec<FpVar<Fr>> = source_outcomes
            .iter()
            .map(|v| {
                FpVar::new_witness(cs.clone(), || {
                    v.ok_or(SynthesisError::AssignmentMissing)
                })
            })
            .collect::<Result<_, _>>()?;

        let weights: Vec<FpVar<Fr>> = source_weights
            .iter()
            .map(|v| {
                FpVar::new_witness(cs.clone(), || {
                    v.ok_or(SynthesisError::AssignmentMissing)
                })
            })
            .collect::<Result<_, _>>()?;

        let included: Vec<FpVar<Fr>> = source_included
            .iter()
            .map(|v| {
                FpVar::new_witness(cs.clone(), || {
                    v.ok_or(SynthesisError::AssignmentMissing)
                })
            })
            .collect::<Result<_, _>>()?;

        let margin_bits: Vec<FpVar<Fr>> = majority_margin_bits
            .iter()
            .map(|v| {
                FpVar::new_witness(cs.clone(), || {
                    v.ok_or(SynthesisError::AssignmentMissing)
                })
            })
            .collect::<Result<_, _>>()?;

        let final_outcome = FpVar::new_input(cs.clone(), || {
            self.final_outcome.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let source_count = FpVar::new_input(cs.clone(), || {
            self.source_count.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let one = FpVar::one();
        let zero = FpVar::zero();

        for outcome in &outcomes {
            outcome.mul_equals(&(outcome - &one), &zero)?;
        }

        for flag in &included {
            flag.mul_equals(&(flag - &one), &zero)?;
        }

        let mut counted = FpVar::zero();
        for flag in &included {
            counted += flag;
        }
        counted.enforce_equal(&source_count)?;

        let mut yes_weight = FpVar::zero();
        let mut total_weight = FpVar::zero();

        for ((outcome, weight), flag) in
            outcomes.iter().zip(weights.iter()).zip(included.iter())
        {
            let effective = weight * flag;
            total_weight += &effective;
            yes_weight += &(outcome * &effective);
        }

        let mut margin = FpVar::zero();
        for (bit_index, bit) in margin_bits.iter().enumerate() {
            bit.mul_equals(&(bit - &one), &zero)?;
            let weight = FpVar::constant(Fr::from(1u64 << bit_index));
            margin += &(bit * &weight);
        }

        let two_yes = &yes_weight + &yes_weight;
        let diff = &two_yes - &total_weight;
        let yes_branch = &final_outcome * (&diff - &margin - &one);
        let no_branch =
            (&one - &final_outcome) * (&total_weight - &two_yes - &margin);
        (yes_branch + no_branch).enforce_equal(&zero)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;

    fn padded_three_yes() -> OracleCircuit {
        let mut source_outcomes = vec![Some(Fr::from(1u64)); 3];
        let mut source_weights = vec![
            Some(Fr::from(3u64)),
            Some(Fr::from(4u64)),
            Some(Fr::from(3u64)),
        ];
        let mut source_included = vec![Some(Fr::from(1u64)); 3];
        source_outcomes.resize(MAX_SOURCES, Some(Fr::from(0u64)));
        source_weights.resize(MAX_SOURCES, Some(Fr::from(0u64)));
        source_included.resize(MAX_SOURCES, Some(Fr::from(0u64)));
        OracleCircuit {
            source_outcomes,
            source_weights,
            source_included,
            majority_margin_bits: {
                let mut bits = vec![Some(Fr::from(0u64)); MAJORITY_MARGIN_BITS];
                bits[0] = Some(Fr::from(1u64));
                bits[3] = Some(Fr::from(1u64));
                bits
            },
            final_outcome: Some(Fr::from(1u64)),
            source_count: Some(Fr::from(3u64)),
        }
    }

    #[test]
    fn circuit_satisfied_for_three_yes_sources() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let circuit = padded_three_yes();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
}
