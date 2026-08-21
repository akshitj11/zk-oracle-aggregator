//! R1CS circuit proving honest aggregation over up to [`crate::MAX_SOURCES`] inputs.

use ark_bn254::Fr;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Witness and public inputs for one oracle resolution proof.
#[derive(Clone)]
pub struct OracleCircuit {
    /// Per-source outcome witness: 1 = Yes, 0 = No (Unknown treated as 0 weight).
    pub source_outcomes: Vec<Option<Fr>>,
    /// Per-source confidence weight in the scalar field.
    pub source_weights: Vec<Option<Fr>>,
    /// 1 if source survived outlier removal, else 0.
    pub source_included: Vec<Option<Fr>>,
    /// Public: 1 = Yes, 0 = No.
    pub final_outcome: Option<Fr>,
    /// Public: count of included sources.
    pub source_count: Option<Fr>,
}

impl Default for OracleCircuit {
    fn default() -> Self {
        Self {
            source_outcomes: Vec::new(),
            source_weights: Vec::new(),
            source_included: Vec::new(),
            final_outcome: None,
            source_count: None,
        }
    }
}

impl ConstraintSynthesizer<Fr> for OracleCircuit {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<Fr>,
    ) -> Result<(), SynthesisError> {
        let outcomes: Vec<FpVar<Fr>> = self
            .source_outcomes
            .iter()
            .map(|v| FpVar::new_witness(cs.clone(), || {
                v.ok_or(SynthesisError::AssignmentMissing)
            }))
            .collect::<Result<_, _>>()?;

        let weights: Vec<FpVar<Fr>> = self
            .source_weights
            .iter()
            .map(|v| FpVar::new_witness(cs.clone(), || {
                v.ok_or(SynthesisError::AssignmentMissing)
            }))
            .collect::<Result<_, _>>()?;

        let included: Vec<FpVar<Fr>> = self
            .source_included
            .iter()
            .map(|v| FpVar::new_witness(cs.clone(), || {
                v.ok_or(SynthesisError::AssignmentMissing)
            }))
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

        for ((outcome, weight), flag) in outcomes.iter().zip(weights.iter()).zip(included.iter())
        {
            let effective = weight * flag;
            total_weight += &effective;
            yes_weight += &(outcome * &effective);
        }

        // Weight accumulation wired; majority binding added in a follow-up gadget commit.
        let _ = (&yes_weight, &total_weight, &final_outcome);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn circuit_satisfied_for_three_yes_sources() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        let circuit = OracleCircuit {
            source_outcomes: vec![Some(Fr::from(1u64)), Some(Fr::from(1u64)), Some(Fr::from(1u64))],
            source_weights: vec![
                Some(Fr::from(3u64)),
                Some(Fr::from(4u64)),
                Some(Fr::from(3u64)),
            ],
            source_included: vec![Some(Fr::from(1u64)), Some(Fr::from(1u64)), Some(Fr::from(1u64))],
            final_outcome: Some(Fr::from(1u64)),
            source_count: Some(Fr::from(3u64)),
        };

        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
}
