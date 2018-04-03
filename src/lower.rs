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

    crate fn for_each_goto_fact<E>(
        &self,
        mut op: impl FnMut(&str, &str) -> Result<(), E>,
    ) -> Result<(), E> {
        for block in &self.blocks {
            let term_point = &format!("{}/{}", block.name, block.statements.len());
            for goto in &block.goto {
                op(term_point, &format!("{}/0", goto))?;
            }
        }
        Ok(())
    }

    crate fn for_each_region_live_on_entry_fact<E>(
        &self,
        mut op: impl FnMut(&str, &str) -> Result<(), E>,
    ) -> Result<(), E> {
        for block in &self.blocks {
            for (index, statement) in block.statements.iter().enumerate() {
                let point = &format!("{}/{}", block.name, index);
                for effect in &statement.effects {
                    if let Effect::LiveOnEntry { region } = effect {
                        op(region, point)?;
                    }
                }
            }
        }
        Ok(())
    }

    crate fn for_each_killed_fact<E>(
        &self,
        mut op: impl FnMut(&str, &str) -> Result<(), E>,
    ) -> Result<(), E> {
        for block in &self.blocks {
            for (index, statement) in block.statements.iter().enumerate() {
                let point = &format!("{}/{}", block.name, index);
                for effect in &statement.effects {
                    if let Effect::Kill { borrow } = effect {
                        op(borrow, point)?;
                    }
                }
            }
        }
        Ok(())
    }

    crate fn for_each_outlives_fact<E>(
        &self,
        mut op: impl FnMut(&str, &str, &str, &str) -> Result<(), E>,
    ) -> Result<(), E> {
        for block in &self.blocks {
            for (index, statement) in block.statements.iter().enumerate() {
                let point = &format!("{}/{}", block.name, index);
                let successor_point = &format!("{}/{}", block.name, index + 1);
                for effect in &statement.effects {
                    if let Effect::Outlives { time, a, b } = effect {
                        match time {
                            OutlivesTime::Pre => op(point, a, b, point)?,
                            OutlivesTime::Post => op(point, a, b, successor_point)?,
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
