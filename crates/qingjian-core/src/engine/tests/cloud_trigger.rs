//! 自动开关、显式触发、私密输入的待发取消与九宫格的请求拼音回归。

use std::sync::{Arc, Mutex};

use qingjian_dictionary::Dictionary;

use super::SAMPLE;
use crate::engine::{Engine, Prediction, PredictionPolicy, PredictionRequest, Predictor};

#[derive(Default)]
struct Requests {
    submitted: Vec<PredictionRequest>,

    cancellations: usize,
}

struct ManualPredictor(Arc<Mutex<Requests>>);

impl Predictor for ManualPredictor {
    fn policy(&self) -> PredictionPolicy {
        PredictionPolicy {
            automatic: false,
            ..PredictionPolicy::default()
        }
    }

    fn submit(&mut self, request: PredictionRequest) {
        self.0.lock().unwrap().submitted.push(request);
    }

    fn cancel(&mut self) {
        self.0.lock().unwrap().cancellations += 1;
    }

    fn poll(&mut self) -> Option<Prediction> {
        None
    }
}

#[test]
fn manual_mode_cancels_automatic_requests_and_allows_explicit_requests() {
    let requests = Arc::new(Mutex::new(Requests::default()));
    let mut engine = Engine::new(Dictionary::parse(SAMPLE).unwrap())
        .with_predictor(Box::new(ManualPredictor(requests.clone())));
    engine.set_input("kaifa");
    let before = requests.lock().unwrap().cancellations;
    assert_eq!(engine.request_prediction(None, &[]), None);
    assert_eq!(requests.lock().unwrap().cancellations, before + 1);
    assert!(requests.lock().unwrap().submitted.is_empty());
    assert!(engine.request_manual_prediction(None, &[]).is_some());
    assert!(requests.lock().unwrap().submitted[0].manual);
    assert!(engine.request_translation("开发").is_some());
    assert!(requests.lock().unwrap().submitted[1].manual);

    let before = requests.lock().unwrap().cancellations;
    engine.set_private(true);
    assert_eq!(requests.lock().unwrap().cancellations, before + 1);
    assert_eq!(engine.request_manual_prediction(None, &[]), None);
    assert_eq!(requests.lock().unwrap().submitted.len(), 2);
    engine.set_private(false);
    engine.discard_input();
    assert!(requests.lock().unwrap().cancellations > before + 1);
}

struct AutomaticPredictor(Arc<Mutex<Requests>>);

impl Predictor for AutomaticPredictor {
    fn policy(&self) -> PredictionPolicy {
        PredictionPolicy::default()
    }

    fn submit(&mut self, request: PredictionRequest) {
        self.0.lock().unwrap().submitted.push(request);
    }

    fn cancel(&mut self) {
        self.0.lock().unwrap().cancellations += 1;
    }

    fn poll(&mut self) -> Option<Prediction> {
        None
    }
}

#[test]
fn nine_key_asks_with_the_pinyin_line() {
    let requests = Arc::new(Mutex::new(Requests::default()));
    let mut engine = Engine::new(Dictionary::parse(SAMPLE).unwrap())
        .with_predictor(Box::new(AutomaticPredictor(requests.clone())));
    engine.set_nine_key(true);
    engine.set_input("52432");
    // 没查过这段数字不知道拼音，不发
    assert_eq!(engine.request_prediction(None, &[]), None);
    engine.query().unwrap();
    assert!(engine.request_prediction(None, &[]).is_some());
    let submitted = &requests.lock().unwrap().submitted;
    assert_eq!(submitted[0].letters, "kaifa");
    assert_eq!(submitted[0].syllables, 2);
}
