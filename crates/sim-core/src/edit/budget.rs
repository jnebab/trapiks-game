use super::EditError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Budget {
    pub limit: Option<i64>,
    pub spent: i64,
}

impl Budget {
    pub fn check(&self, cost: i64) -> Result<(), EditError> {
        match self.limit {
            Some(limit) if self.spent.saturating_add(cost) > limit => {
                Err(EditError::InsufficientBudget)
            }
            _ => Ok(()),
        }
    }
}
