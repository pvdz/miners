
pub fn progress_bar(bar_max_width: usize, cur_cooldown: f32, max_cooldown: f32, lte: bool) -> String {
  if max_cooldown == 0.0 {
    return "".to_string();
  }

  assert!(cur_cooldown >= 0.0, "cooldown should not be negative {:?} {:?}", cur_cooldown, max_cooldown);
  assert!(!lte || cur_cooldown <= max_cooldown, "cooldown should not exceed the max unless explicitly allowed to do so... {:?} {:?} {}", cur_cooldown, max_cooldown, lte);

  let progress = ((cur_cooldown / max_cooldown) * bar_max_width as f32).min(bar_max_width as f32) as usize;
  let remaining = bar_max_width - progress;
  return format!(
    "[{}{}] ({: >3}%)",
    std::iter::repeat('|').take(progress).collect::<String>(),
    std::iter::repeat('-').take(remaining).collect::<String>(),
    ((cur_cooldown / max_cooldown) * 100.0) as i32,
  )
}
