use crate::edit::vehicles::{
    HeldMovement, RouteDamage, record_movements, remap_movements, settle_links,
};
use crate::edit::{
    Budget, CommandResult, Edit, EditCommand, EditError, EditOutcome, Outcome, Prepared,
    QuoteOutcome, Scope, UndoEntry, prepare,
};

use std::collections::{BTreeSet, VecDeque};

use super::{RerouteCounts, Sim};

pub(super) struct EditState {
    pub commands: VecDeque<(u32, EditCommand)>,
    pub next_seq: u32,
    pub results: Vec<CommandResult>,
    pub undo: Vec<UndoEntry>,
    pub budget: Budget,
    pub flagged: BTreeSet<u32>,
    pub reroutes: RerouteCounts,
    pub log: Vec<EditCommand>,
}

impl EditState {
    pub fn new(limit: Option<i64>) -> EditState {
        EditState {
            commands: VecDeque::new(),
            next_seq: 0,
            results: Vec::new(),
            undo: Vec::new(),
            budget: Budget { limit, spent: 0 },
            flagged: BTreeSet::new(),
            reroutes: RerouteCounts::default(),
            log: Vec::new(),
        }
    }

    fn record(&mut self, command: EditCommand) {
        let is_demand = |c: &EditCommand| matches!(c, EditCommand::SetDemand { .. });
        if !is_demand(&command) {
            self.log.push(command);
            return;
        }
        match self.log.iter_mut().find(|c| is_demand(c)) {
            Some(slot) => *slot = command,
            None => self.log.push(command),
        }
    }
}

impl Sim {
    pub fn command_log(&self) -> &[EditCommand] {
        &self.edits.log
    }

    pub fn enqueue(&mut self, command: EditCommand) -> u32 {
        let seq = self.edits.next_seq;
        self.edits.next_seq = self.edits.next_seq.wrapping_add(1);
        self.edits.commands.push_back((seq, command));
        seq
    }

    pub fn take_results(&mut self) -> Vec<CommandResult> {
        std::mem::take(&mut self.edits.results)
    }

    pub fn quote(&self, command: &EditCommand) -> QuoteOutcome {
        match self.quote_cost(command) {
            Ok(cost) => QuoteOutcome::Ok(cost),
            Err(error) => QuoteOutcome::Err(error),
        }
    }

    fn quote_cost(&self, command: &EditCommand) -> Result<i64, EditError> {
        match self.prepare(command)? {
            Prepared::Edit { cost, .. } => self.edits.budget.check(cost).map(|()| cost),
            Prepared::Undo => Ok(self.edits.undo.last().map_or(0, |entry| -entry.cost)),
            Prepared::Demand { .. } => Ok(0),
        }
    }

    pub fn set_budget_limit(&mut self, limit: Option<i64>) {
        self.edits.budget.limit = limit;
    }

    pub fn budget(&self) -> Budget {
        self.edits.budget
    }

    pub fn undo_depth(&self) -> usize {
        self.edits.undo.len()
    }

    fn prepare(&self, command: &EditCommand) -> Result<Prepared, EditError> {
        prepare(&self.network, command, !self.edits.undo.is_empty())
    }

    pub fn apply_queued(&mut self) {
        while let Some((seq, command)) = self.edits.commands.pop_front() {
            let outcome = match self.execute(&command) {
                Ok(outcome) => {
                    self.edits.record(command);
                    Outcome::Ok(outcome)
                }
                Err(error) => Outcome::Err(error),
            };
            self.edits.results.push(CommandResult { seq, outcome });
        }
    }

    fn execute(&mut self, command: &EditCommand) -> Result<EditOutcome, EditError> {
        match self.prepare(command)? {
            Prepared::Edit { edit, cost } => {
                self.edits.budget.check(cost)?;
                Ok(self.perform(&edit, cost))
            }
            Prepared::Undo => self.undo_last(),
            Prepared::Demand { vehicles_per_hour } => {
                self.set_demand(vehicles_per_hour);
                Ok(EditOutcome::default())
            }
        }
    }

    fn perform(&mut self, edit: &Edit, cost: i64) -> EditOutcome {
        let (inverse, outcome) = self.apply_edit(edit, cost);
        self.edits.undo.push(UndoEntry { inverse, cost });
        self.edits.budget.spent += cost;
        outcome
    }

    fn undo_last(&mut self) -> Result<EditOutcome, EditError> {
        let entry = self.edits.undo.pop().ok_or(EditError::NothingToUndo)?;
        let (_, outcome) = self.apply_edit(&entry.inverse, -entry.cost);
        self.edits.budget.spent -= entry.cost;
        Ok(outcome)
    }

    fn apply_edit(&mut self, edit: &Edit, cost: i64) -> (Edit, EditOutcome) {
        match edit {
            Edit::BuildFlyover { node, through } => {
                self.apply_flyover(edit, (*node, *through), cost)
            }
            Edit::UndoFlyover(undo) => self.undo_flyover(undo, cost),
            _ => self.apply_plain(edit, cost),
        }
    }

    fn apply_plain(&mut self, edit: &Edit, cost: i64) -> (Edit, EditOutcome) {
        let scope = edit.scope(&self.network);
        let held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        let inverse = edit.apply(&mut self.network);
        self.network.commit_edit(&scope.invalidated, scope.signals);
        self.settle_vehicles(&scope.roads, &scope.invalidated, &held);
        (inverse, outcome(scope, cost))
    }

    pub(super) fn settle_vehicles(
        &mut self,
        roads: &[u32],
        invalidated: &[u32],
        held: &[HeldMovement],
    ) {
        let mut stranded = settle_links(&self.network, &mut self.vehicles, invalidated);
        stranded.extend(remap_movements(&mut self.network, &mut self.vehicles, held));
        stranded.sort_unstable();
        for slot in stranded {
            self.strand(slot);
        }
        let damage = RouteDamage::new(&self.network, roads, invalidated);
        self.flag_damaged(&damage);
    }
}

pub(super) fn outcome(scope: Scope, cost: i64) -> EditOutcome {
    EditOutcome {
        cost,
        changed_roads: scope.roads,
        changed_nodes: scope.nodes,
    }
}
