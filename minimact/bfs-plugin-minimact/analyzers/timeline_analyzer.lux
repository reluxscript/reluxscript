/**
 * Timeline Analyzer Module
 *
 * Detects and analyzes @minimact/timeline usage in components:
 * - useTimeline() calls → [Timeline] attribute
 * - useTimelineState() bindings → [TimelineStateBinding] attributes
 * - timeline.keyframes() calls → [TimelineKeyframe] attributes
 * - Timeline configuration (duration, repeat, easing, etc.)
 */

/**
 * Timeline metadata
 */
pub struct TimelineMetadata {
    pub timeline_id: Str,
    pub variable_name: Str,
    pub duration: i32,
    pub repeat: bool,
    pub repeat_count: i32,
    pub easing: Str,
    pub auto_play: bool,
    pub state_bindings: HashMap<Str, StateBinding>,
    pub keyframes: Vec<Keyframe>,
    pub control_methods: Vec<ControlMethod>,
}

/**
 * State binding information
 */
pub struct StateBinding {
    pub state_key: Str,
    pub setter_name: Str,
    pub interpolate: bool,
    pub state_type: Str,
}

/**
 * Keyframe information
 */
pub struct Keyframe {
    pub time: i32,  // in milliseconds
    pub state: HashMap<Str, KeyframeValue>,
    pub label: Option<Str>,
    pub easing: Option<Str>,
}

/**
 * Keyframe state value
 */
pub enum KeyframeValue {
    Number(f64),
    String(Str),
    Boolean(bool),
    Null,
}

/**
 * Control method call
 */
pub struct ControlMethod {
    pub method: Str,  // 'play', 'pause', 'stop', 'seek', 'reverse'
    pub arguments: Vec<Option<Str>>,
}

/**
 * Validation result
 */
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<Str>,
}

/**
 * Analyze timeline usage in a component
 *
 * @param component_func - Function declaration of component
 * @param component_name - Component name
 * @returns Timeline metadata or None
 */
pub fn analyze_timeline(component_func: &FunctionDeclaration, component_name: &Str) -> Option<TimelineMetadata> {
    let body = component_func.body.as_ref()?;
    let mut timeline: Option<TimelineMetadata> = None;

    // 1. Find useTimeline() call
    for stmt in &body.body {
        if let Some(found_timeline) = find_use_timeline(stmt, component_name) {
            timeline = Some(found_timeline);
            break;
        }
    }

    if timeline.is_none() {
        return None;
    }

    let mut timeline_meta = timeline.unwrap();

    // 2. Find useTimelineState() calls
    extract_timeline_state_bindings(body, &mut timeline_meta);

    // 3. Find timeline.keyframes() or timeline.keyframe() calls
    extract_timeline_keyframes(body, &mut timeline_meta);

    // 4. Validate timeline
    let validation = validate_timeline(&timeline_meta);
    if !validation.valid {
        // In a real implementation, we would log errors
        // For now, we still return the timeline
    }

    Some(timeline_meta)
}

/**
 * Find useTimeline() call in statement
 */
fn find_use_timeline(stmt: &Statement, component_name: &Str) -> Option<TimelineMetadata> {
    if let Statement::VariableDeclaration(ref var_decl) = stmt {
        for declarator in &var_decl.declarations {
            if let Some(ref init) = declarator.init {
                if let Expression::CallExpression(ref call) = init {
                    if let Expression::Identifier(ref callee) = call.callee {
                        if callee.name == "useTimeline" {
                            // Found useTimeline() call
                            let variable_name = if let Pattern::Identifier(ref id) = declarator.id {
                                id.name.clone()
                            } else {
                                "timeline".to_string()
                            };

                            let mut timeline = TimelineMetadata {
                                timeline_id: format!("{}_Timeline", component_name),
                                variable_name,
                                duration: 0,
                                repeat: false,
                                repeat_count: -1,
                                easing: "linear".to_string(),
                                auto_play: false,
                                state_bindings: HashMap::new(),
                                keyframes: vec![],
                                control_methods: vec![],
                            };

                            // Extract config object (first argument)
                            if !call.arguments.is_empty() {
                                if let Expression::ObjectExpression(ref config) = call.arguments[0] {
                                    extract_timeline_config(config, &mut timeline);
                                }
                            }

                            // Extract optional name argument (second argument)
                            if call.arguments.len() >= 2 {
                                if let Expression::StringLiteral(ref name_lit) = call.arguments[1] {
                                    timeline.timeline_id = format!("{}_{}", component_name, name_lit.value);
                                }
                            }

                            return Some(timeline);
                        }
                    }
                }
            }
        }
    }

    None
}

/**
 * Extract timeline configuration from config object
 */
fn extract_timeline_config(config: &ObjectExpression, timeline: &mut TimelineMetadata) {
    for prop in &config.properties {
        if let ObjectProperty::Property(ref obj_prop) = prop {
            if let Expression::Identifier(ref key) = obj_prop.key {
                let key_name = &key.name;
                let value = &obj_prop.value;

                match key_name.as_str() {
                    "duration" => {
                        if let Expression::NumericLiteral(ref num) = value {
                            timeline.duration = num.value as i32;
                        }
                    }
                    "repeat" => {
                        if let Expression::BooleanLiteral(ref bool_lit) = value {
                            timeline.repeat = bool_lit.value;
                        }
                    }
                    "repeatCount" => {
                        if let Expression::NumericLiteral(ref num) = value {
                            timeline.repeat_count = num.value as i32;
                        }
                    }
                    "easing" => {
                        if let Expression::StringLiteral(ref str_lit) = value {
                            timeline.easing = str_lit.value.clone();
                        }
                    }
                    "autoPlay" => {
                        if let Expression::BooleanLiteral(ref bool_lit) = value {
                            timeline.auto_play = bool_lit.value;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/**
 * Extract state bindings from useTimelineState() calls
 */
fn extract_timeline_state_bindings(body: &BlockStatement, timeline: &mut TimelineMetadata) {
    for stmt in &body.body {
        extract_state_binding_from_statement(stmt, timeline);
    }
}

/**
 * Extract state binding from a statement
 */
fn extract_state_binding_from_statement(stmt: &Statement, timeline: &mut TimelineMetadata) {
    match stmt {
        Statement::ExpressionStatement(ref expr_stmt) => {
            if let Expression::CallExpression(ref call) = expr_stmt.expression {
                if let Expression::Identifier(ref callee) = call.callee {
                    if callee.name == "useTimelineState" {
                        extract_state_binding_from_call(call, timeline);
                    }
                }
            }
        }

        Statement::VariableDeclaration(ref var_decl) => {
            for declarator in &var_decl.declarations {
                if let Some(ref init) = declarator.init {
                    if let Expression::CallExpression(ref call) = init {
                        if let Expression::Identifier(ref callee) = call.callee {
                            if callee.name == "useTimelineState" {
                                extract_state_binding_from_call(call, timeline);
                            }
                        }
                    }
                }
            }
        }

        _ => {}
    }
}

/**
 * Extract state binding from useTimelineState() call
 * useTimelineState(timeline, 'stateKey', setter, interpolate)
 */
fn extract_state_binding_from_call(call: &CallExpression, timeline: &mut TimelineMetadata) {
    if call.arguments.len() >= 3 {
        // Argument 1: state key (string)
        let state_key = if let Expression::StringLiteral(ref str_lit) = call.arguments[1] {
            str_lit.value.clone()
        } else {
            return;
        };

        // Argument 2: setter (identifier)
        let setter_name = if let Expression::Identifier(ref id) = call.arguments[2] {
            id.name.clone()
        } else {
            return;
        };

        // Argument 3 (optional): interpolate (boolean)
        let interpolate = if call.arguments.len() >= 4 {
            if let Expression::BooleanLiteral(ref bool_lit) = call.arguments[3] {
                bool_lit.value
            } else {
                false
            }
        } else {
            false
        };

        let binding = StateBinding {
            state_key: state_key.clone(),
            setter_name,
            interpolate,
            state_type: "unknown".to_string(),
        };

        timeline.state_bindings.insert(state_key, binding);
    }
}

/**
 * Extract keyframes from timeline methods
 */
fn extract_timeline_keyframes(body: &BlockStatement, timeline: &mut TimelineMetadata) {
    for stmt in &body.body {
        extract_keyframes_from_statement(stmt, timeline);
    }
}

/**
 * Extract keyframes from a statement
 */
fn extract_keyframes_from_statement(stmt: &Statement, timeline: &mut TimelineMetadata) {
    if let Statement::ExpressionStatement(ref expr_stmt) = stmt {
        if let Expression::CallExpression(ref call) = expr_stmt.expression {
            if let Expression::MemberExpression(ref member) = call.callee {
                if let Expression::Identifier(ref obj) = member.object {
                    if obj.name == timeline.variable_name {
                        if let Expression::Identifier(ref prop) = member.property {
                            match prop.name.as_str() {
                                "keyframes" => {
                                    extract_keyframes_array(call, timeline);
                                }
                                "keyframe" => {
                                    extract_single_keyframe(call, timeline);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

/**
 * Extract keyframes from timeline.keyframes([...]) call
 */
fn extract_keyframes_array(call: &CallExpression, timeline: &mut TimelineMetadata) {
    if call.arguments.is_empty() {
        return;
    }

    if let Expression::ArrayExpression(ref array) = call.arguments[0] {
        for element in &array.elements {
            if let Some(ref elem) = element {
                if let Expression::ObjectExpression(ref obj) = elem {
                    if let Some(keyframe) = parse_keyframe_object(obj) {
                        timeline.keyframes.push(keyframe);
                    }
                }
            }
        }
    }
}

/**
 * Extract single keyframe from timeline.keyframe(time, state) call
 */
fn extract_single_keyframe(call: &CallExpression, timeline: &mut TimelineMetadata) {
    if call.arguments.len() < 2 {
        return;
    }

    let time = if let Expression::NumericLiteral(ref num) = call.arguments[0] {
        num.value as i32
    } else {
        return;
    };

    if let Expression::ObjectExpression(ref state_obj) = call.arguments[1] {
        if let Some(mut keyframe) = parse_keyframe_object(state_obj) {
            keyframe.time = time;
            timeline.keyframes.push(keyframe);
        }
    }
}

/**
 * Parse keyframe object: { time: 0, state: { count: 0 }, label?: '', easing?: '' }
 */
fn parse_keyframe_object(obj: &ObjectExpression) -> Option<Keyframe> {
    let mut keyframe = Keyframe {
        time: 0,
        state: HashMap::new(),
        label: None,
        easing: None,
    };

    for prop in &obj.properties {
        if let ObjectProperty::Property(ref obj_prop) = prop {
            if let Expression::Identifier(ref key) = obj_prop.key {
                let key_name = &key.name;
                let value = &obj_prop.value;

                match key_name.as_str() {
                    "time" => {
                        if let Expression::NumericLiteral(ref num) = value {
                            keyframe.time = num.value as i32;
                        }
                    }
                    "state" => {
                        if let Expression::ObjectExpression(ref state_obj) = value {
                            keyframe.state = parse_state_object(state_obj);
                        }
                    }
                    "label" => {
                        if let Expression::StringLiteral(ref str_lit) = value {
                            keyframe.label = Some(str_lit.value.clone());
                        }
                    }
                    "easing" => {
                        if let Expression::StringLiteral(ref str_lit) = value {
                            keyframe.easing = Some(str_lit.value.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Some(keyframe)
}

/**
 * Parse state object into HashMap
 */
fn parse_state_object(obj: &ObjectExpression) -> HashMap<Str, KeyframeValue> {
    let mut state = HashMap::new();

    for prop in &obj.properties {
        if let ObjectProperty::Property(ref obj_prop) = prop {
            if let Expression::Identifier(ref key) = obj_prop.key {
                let key_name = key.name.clone();
                let value = &obj_prop.value;

                let kf_value = match value {
                    Expression::NumericLiteral(ref num) => KeyframeValue::Number(num.value),
                    Expression::StringLiteral(ref str_lit) => KeyframeValue::String(str_lit.value.clone()),
                    Expression::BooleanLiteral(ref bool_lit) => KeyframeValue::Boolean(bool_lit.value),
                    Expression::NullLiteral => KeyframeValue::Null,
                    Expression::UnaryExpression(ref unary) => {
                        // Handle negative numbers
                        if unary.operator == "-" {
                            if let Expression::NumericLiteral(ref num) = unary.argument {
                                KeyframeValue::Number(-num.value)
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

                state.insert(key_name, kf_value);
            }
        }
    }

    state
}

/**
 * Validate timeline definition
 */
pub fn validate_timeline(timeline: &TimelineMetadata) -> ValidationResult {
    let mut errors = vec![];

    // Check duration
    if timeline.duration <= 0 {
        errors.push(format!("Timeline duration must be positive (got {}ms)", timeline.duration));
    }

    // Check keyframe timing
    for (index, keyframe) in timeline.keyframes.iter().enumerate() {
        // Check time ordering
        if index > 0 {
            let prev_time = timeline.keyframes[index - 1].time;
            if keyframe.time <= prev_time {
                errors.push(format!(
                    "Keyframe times must be ascending. Keyframe at index {} ({}ms) is not after previous ({}ms).",
                    index, keyframe.time, prev_time
                ));
            }
        }

        // Check time within duration
        if keyframe.time > timeline.duration {
            errors.push(format!(
                "Keyframe at {}ms exceeds timeline duration ({}ms).",
                keyframe.time, timeline.duration
            ));
        }

        // Check state keys are bound
        for (state_key, _) in &keyframe.state {
            if !timeline.state_bindings.contains_key(state_key) {
                errors.push(format!(
                    "Keyframe at {}ms references unbound state '{}'. Add useTimelineState(timeline, '{}', ...).",
                    keyframe.time, state_key, state_key
                ));
            }
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}
