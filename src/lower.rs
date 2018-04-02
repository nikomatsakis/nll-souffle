use ir::*;

impl Input {
    crate fn for_each_borrow_region_fact<E>(
        &self,
        mut op: impl FnMut(&str, &str, &str) -> Result<(), E>,
    ) -> Result<(), E> {
        for block in &self.blocks {
            for (index, statement) in block.statements.iter().enumerate() {
                let point = &format!("{}/{}", block.name, index);
                for effect in &statement.effects {
                    if let Effect::Borrow { borrow, region } = effect {
                        op(region, borrow, point)?;
                    }
                }
            }
        }
        Ok(())
    }

    crate fn for_each_next_statement_fact<E>(
        &self,
        mut op: impl FnMut(&str, &str) -> Result<(), E>,
    ) -> Result<(), E> {
        for block in &self.blocks {
            let mut prev_point: Option<String> = None;
            for index in 0..block.statements.len() {
                let point = format!("{}/{}", block.name, index);
                if let Some(prev_point) = prev_point {
                    op(&prev_point, &point)?;
                }
                prev_point = Some(point);
            }

            let term_point = format!("{}/{}", block.name, block.statements.len());
            if let Some(prev_point) = prev_point {
                op(&prev_point, &term_point)?;
            }
        }

        Ok(())
    }
}
