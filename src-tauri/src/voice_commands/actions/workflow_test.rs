use super::*;
use crate::voice_commands::executor::{Action, ActionError, ActionErrorCode, ActionResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

/// Mock action for testing
struct MockAction {
    result: Result<ActionResult, ActionError>,
    execution_count: AtomicUsize,
    execution_order: Arc<Mutex<Vec<usize>>>,
    id: usize,
}

impl MockAction {
    fn new_success(id: usize, message: &str, order: Arc<Mutex<Vec<usize>>>) -> Self {
        Self {
            result: Ok(ActionResult {
                message: message.to_string(),
                data: None,
            }),
            execution_count: AtomicUsize::new(0),
            execution_order: order,
            id,
        }
    }

    fn new_failure(id: usize, code: ActionErrorCode, message: &str, order: Arc<Mutex<Vec<usize>>>) -> Self {
        Self {
            result: Err(ActionError {
                code,
                message: message.to_string(),
            }),
            execution_count: AtomicUsize::new(0),
            execution_order: order,
            id,
        }
    }
}

#[async_trait]
impl Action for MockAction {
    async fn execute(&self, _parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        self.execution_order.lock().await.push(self.id);
        self.result.clone()
    }
}

fn create_dispatcher_with_mocks(
    open_app: Arc<dyn Action>,
    type_text: Arc<dyn Action>,
) -> Arc<ActionDispatcher> {
    // Use a simple mock for other actions
    let order = Arc::new(Mutex::new(Vec::new()));
    Arc::new(ActionDispatcher::with_actions(
        open_app,
        type_text,
        Arc::new(MockAction::new_success(0, "System control", order.clone())),
        Arc::new(MockAction::new_success(0, "Workflow", order.clone())),
        Arc::new(MockAction::new_success(0, "Custom", order.clone())),
    ))
}

fn steps_json(steps: Vec<WorkflowStep>) -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("steps".to_string(), serde_json::to_string(&steps).unwrap());
    params
}

#[tokio::test]
async fn test_workflow_with_2_steps_executes_in_order() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_success(1, "Step 1 done", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2 done", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let steps = vec![
        WorkflowStep {
            action_type: "open_app".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(0),
        },
        WorkflowStep {
            action_type: "type_text".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(0),
        },
    ];

    let result = workflow.execute(&steps_json(steps)).await;

    assert!(result.is_ok());
    let order_vec = order.lock().await;
    assert_eq!(order_vec.as_slice(), &[1, 2]);
}

#[tokio::test]
async fn test_step_1_failure_stops_workflow() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_failure(1, ActionErrorCode::ExecutionError, "Step 1 failed", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2 done", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let steps = vec![
        WorkflowStep {
            action_type: "open_app".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(0),
        },
        WorkflowStep {
            action_type: "type_text".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(0),
        },
    ];

    let result = workflow.execute(&steps_json(steps)).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::StepFailed);
    assert!(error.message.contains("step 1"));

    // Step 2 should not have been executed
    let order_vec = order.lock().await;
    assert_eq!(order_vec.as_slice(), &[1]);
}

#[tokio::test]
async fn test_all_steps_success_returns_aggregated_result() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_success(1, "Step 1 done", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2 done", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let steps = vec![
        WorkflowStep {
            action_type: "open_app".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(0),
        },
        WorkflowStep {
            action_type: "type_text".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(0),
        },
    ];

    let result = workflow.execute(&steps_json(steps)).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.message.contains("2 steps executed"));

    let data = result.data.unwrap();
    assert_eq!(data["steps_executed"], 2);
}

#[tokio::test]
async fn test_delay_between_steps_respected() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_success(1, "Step 1", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let steps = vec![
        WorkflowStep {
            action_type: "open_app".to_string(),
            parameters: HashMap::new(),
            delay_ms: Some(50), // 50ms delay after step 1
        },
        WorkflowStep {
            action_type: "type_text".to_string(),
            parameters: HashMap::new(),
            delay_ms: None,
        },
    ];

    let start = Instant::now();
    let result = workflow.execute(&steps_json(steps)).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Should take at least 50ms due to delay
    assert!(elapsed.as_millis() >= 40, "Elapsed: {}ms", elapsed.as_millis());
}

#[tokio::test]
async fn test_empty_workflow_returns_success() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_success(1, "Step 1", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let steps: Vec<WorkflowStep> = vec![];
    let result = workflow.execute(&steps_json(steps)).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.message.contains("no steps"));

    let data = result.data.unwrap();
    assert_eq!(data["steps_executed"], 0);
}

#[tokio::test]
async fn test_workflow_with_5_steps_executes_all_sequentially() {
    let order = Arc::new(Mutex::new(Vec::new()));

    // Create unique mock actions
    struct SequentialMock {
        id: usize,
        order: Arc<Mutex<Vec<usize>>>,
    }

    #[async_trait]
    impl Action for SequentialMock {
        async fn execute(&self, _params: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
            self.order.lock().await.push(self.id);
            Ok(ActionResult {
                message: format!("Step {}", self.id),
                data: None,
            })
        }
    }

    let open_app = Arc::new(SequentialMock { id: 1, order: order.clone() });
    let type_text = Arc::new(SequentialMock { id: 2, order: order.clone() });
    let system = Arc::new(SequentialMock { id: 3, order: order.clone() });
    let workflow_action = Arc::new(SequentialMock { id: 4, order: order.clone() });
    let custom = Arc::new(SequentialMock { id: 5, order: order.clone() });

    let dispatcher = Arc::new(ActionDispatcher::with_actions(
        open_app, type_text, system, workflow_action, custom
    ));
    let workflow = WorkflowAction::new(dispatcher);

    let steps = vec![
        WorkflowStep { action_type: "open_app".to_string(), parameters: HashMap::new(), delay_ms: Some(0) },
        WorkflowStep { action_type: "type_text".to_string(), parameters: HashMap::new(), delay_ms: Some(0) },
        WorkflowStep { action_type: "system_control".to_string(), parameters: HashMap::new(), delay_ms: Some(0) },
        WorkflowStep { action_type: "workflow".to_string(), parameters: HashMap::new(), delay_ms: Some(0) },
        WorkflowStep { action_type: "custom".to_string(), parameters: HashMap::new(), delay_ms: Some(0) },
    ];

    let result = workflow.execute(&steps_json(steps)).await;

    assert!(result.is_ok());
    let order_vec = order.lock().await;
    assert_eq!(order_vec.as_slice(), &[1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_missing_steps_parameter_returns_error() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_success(1, "Step 1", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let result = workflow.execute(&HashMap::new()).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::InvalidParameter);
}

#[tokio::test]
async fn test_invalid_json_returns_parse_error() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mock1 = Arc::new(MockAction::new_success(1, "Step 1", order.clone()));
    let mock2 = Arc::new(MockAction::new_success(2, "Step 2", order.clone()));

    let dispatcher = create_dispatcher_with_mocks(mock1, mock2);
    let workflow = WorkflowAction::new(dispatcher);

    let mut params = HashMap::new();
    params.insert("steps".to_string(), "not valid json".to_string());

    let result = workflow.execute(&params).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::ParseError);
}
