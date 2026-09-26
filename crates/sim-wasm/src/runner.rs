use trapiks_sim_core::challenge::{Challenge, ChallengeRun, RunResult, score as score_run};
use trapiks_sim_core::edit::EditCommand;
use wasm_bindgen::prelude::*;

use crate::map_handle::MapHandle;
use crate::shared::{from_js, js_error, to_js};

#[wasm_bindgen]
pub struct ChallengeRunner {
    run: ChallengeRun,
}

#[wasm_bindgen]
impl ChallengeRunner {
    #[wasm_bindgen(constructor)]
    pub fn new(
        map: &MapHandle,
        challenge: JsValue,
        commands: JsValue,
    ) -> Result<ChallengeRunner, JsError> {
        let challenge: Challenge = from_js(challenge)?;
        challenge
            .validate()
            .map_err(|e| js_error(format!("{e:?}")))?;
        let commands: Vec<EditCommand> = from_js(commands)?;
        let run = ChallengeRun::new(&map.loaded().map, &challenge, &commands)
            .map_err(|e| js_error(format!("{e:?}")))?;
        Ok(ChallengeRunner { run })
    }

    pub fn advance(&mut self, max_steps: u32) -> Result<JsValue, JsError> {
        to_js(&self.run.advance(max_steps))
    }
}

#[wasm_bindgen]
pub fn score(
    challenge: JsValue,
    baseline: JsValue,
    after: JsValue,
    cost: f64,
) -> Result<JsValue, JsError> {
    let challenge: Challenge = from_js(challenge)?;
    let baseline: RunResult = from_js(baseline)?;
    let after: RunResult = from_js(after)?;
    to_js(&score_run(&challenge, &baseline, &after, cost as i64))
}
