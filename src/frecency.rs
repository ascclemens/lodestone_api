use chrono::Utc;

const LN_2: f64 = 0.6931471805599453;
const HALF_LIFE: f64 = 1.5 * 24.0 * 60.0 * 60.0;
crate const DECAY: f64 = LN_2 / HALF_LIFE;

crate fn frecency(current: Option<f64>) -> f64 {
  let now_decay = Utc::now().timestamp() as f64 * DECAY;

  let current = match current {
    Some(f) => f,
    None => return now_decay,
  };

  let score = (current - now_decay).exp();

  score.ln_1p() + now_decay
}

// crate fn frecency_score(frecency: f64) -> f64 {
//   let now_decay = Utc::now().timestamp() as f64 * DECAY;

//   (frecency - now_decay).exp()
// }
