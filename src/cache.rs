use std::collections::HashMap;
use std::hash::Hash;

/*
 * past this many answers a cache empties and starts over, so questions
 * that never come back (a clock's text, for example) can't fill memory
 */
const MAX_ANSWERS: usize = 4096;

/*
 * answers worked out on earlier redraws, looked up by the question that
 * gave them; the question must hold everything the answer depends on,
 * or an old answer is handed out after something it used has changed
 */
pub struct Cache<Question, Answer> {
    answers: HashMap<Question, Answer>,
}

impl<Question: Eq + Hash, Answer: Clone> Cache<Question, Answer> {
    pub fn new() -> Self {
        Self {
            answers: HashMap::new(),
        }
    }

    // the answer kept from before, or a new one from `work_out`, kept for next time
    pub fn look_up(&mut self, question: Question, work_out: impl FnOnce() -> Answer) -> Answer {
        if let Some(kept) = self.answers.get(&question) {
            return kept.clone();
        }

        let answer = work_out();

        if self.answers.len() >= MAX_ANSWERS {
            self.answers.clear();
        }

        self.answers.insert(question, answer.clone());

        answer
    }
}

#[cfg(test)]
mod tests {
    use super::Cache;

    #[test]
    fn works_out_each_question_only_once() {
        let mut squares = Cache::new();

        let mut worked_out = 0;

        for _ in 0..3 {
            let answer = squares.look_up(4, || {
                worked_out += 1;

                4 * 4
            });

            assert_eq!(answer, 16);
        }

        assert_eq!(worked_out, 1);
    }

    #[test]
    fn starts_over_when_full() {
        let mut doubles = Cache::new();

        for number in 0..super::MAX_ANSWERS {
            doubles.look_up(number, || number * 2);
        }

        doubles.look_up(super::MAX_ANSWERS, || 0);

        assert_eq!(doubles.answers.len(), 1);
    }
}
