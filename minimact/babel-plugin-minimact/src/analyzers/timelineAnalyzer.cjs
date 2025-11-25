/**
 * Timeline Analyzer Module
 *
 * Detects and analyzes @minimact/timeline usage in components:
 * - useTimeline() calls → [Timeline] attribute
 * - useTimelineState() bindings → [TimelineStateBinding] attributes
 * - timeline.keyframes() calls → [TimelineKeyframe] attributes
 * - Timeline configuration (duration, repeat, easing, etc.)
 */

const t = require('@babel/types');

/**
 * Analyze timeline usage in a component
 *
 * @param {NodePath} componentPath - Path to component function
 * @param {string} componentName - Component name
 * @returns {Object|null} - Timeline metadata or null
 */
function analyzeTimeline(componentPath, componentName) {
  let timeline = null;

  // 1. Find useTimeline() call
  componentPath.traverse({
    VariableDeclarator(path) {
      const init = path.node.init;

      if (
        t.isCallExpression(init) &&
        t.isIdentifier(init.callee) &&
        init.callee.name === 'useTimeline'
      ) {
        // Initialize timeline metadata
        timeline = {
          timelineId: `${componentName}_Timeline`,
          variableName: t.isIdentifier(path.node.id) ? path.node.id.name : 'timeline',
          duration: 0,
          repeat: false,
          repeatCount: -1,
          easing: 'linear',
          autoPlay: false,
          stateBindings: new Map(),
          keyframes: [],
          controlMethods: []
        };

        // Extract config object
        const configArg = init.arguments[0];
        if (t.isObjectExpression(configArg)) {
          extractTimelineConfig(configArg, timeline);
        }

        // Extract optional name argument
        const nameArg = init.arguments[1];
        if (t.isStringLiteral(nameArg)) {
          timeline.timelineId = `${componentName}_${nameArg.value}`;
        }

        console.log(`[Timeline] Detected useTimeline() in ${componentName}: ${timeline.variableName}`);
      }
    }
  });

  if (!timeline) {
    return null; // No timeline found
  }

  // 2. Find useTimelineState() calls
  componentPath.traverse({
    CallExpression(path) {
      if (
        t.isIdentifier(path.node.callee) &&
        path.node.callee.name === 'useTimelineState'
      ) {
        extractStateBinding(path.node, timeline);
      }
    }
  });

  // 3. Find timeline.keyframes() or timeline.keyframe() calls
  componentPath.traverse({
    CallExpression(path) {
      const callee = path.node.callee;

      if (
        t.isMemberExpression(callee) &&
        t.isIdentifier(callee.object) &&
        callee.object.name === timeline.variableName &&
        t.isIdentifier(callee.property)
      ) {
        const methodName = callee.property.name;

        if (methodName === 'keyframes') {
          extractKeyframesArray(path.node, timeline);
        } else if (methodName === 'keyframe') {
          extractSingleKeyframe(path.node, timeline);
        } else if (['play', 'pause', 'stop', 'seek', 'reverse'].includes(methodName)) {
          extractControlMethod(path.node, timeline, methodName);
        }
      }
    }
  });

  // 4. Validate timeline
  const validation = validateTimeline(timeline);
  if (!validation.valid) {
    console.error(`[Timeline] Validation failed for ${componentName}:`);
    validation.errors.forEach(err => console.error(`  - ${err}`));
    return null;
  }

  return timeline;
}

/**
 * Extract timeline configuration from useTimeline() config object
 */
function extractTimelineConfig(config, timeline) {
  config.properties.forEach(prop => {
    if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
      const key = prop.key.name;
      const value = prop.value;

      switch (key) {
        case 'duration':
          if (t.isNumericLiteral(value)) {
            timeline.duration = value.value;
          }
          break;
        case 'repeat':
          if (t.isBooleanLiteral(value)) {
            timeline.repeat = value.value;
          }
          break;
        case 'repeatCount':
          if (t.isNumericLiteral(value)) {
            timeline.repeatCount = value.value;
          }
          break;
        case 'easing':
          if (t.isStringLiteral(value)) {
            timeline.easing = value.value;
          }
          break;
        case 'autoPlay':
          if (t.isBooleanLiteral(value)) {
            timeline.autoPlay = value.value;
          }
          break;
      }
    }
  });

  console.log(`[Timeline] Config: duration=${timeline.duration}ms, repeat=${timeline.repeat}, easing=${timeline.easing}`);
}

/**
 * Extract state binding from useTimelineState() call
 * useTimelineState(timeline, 'stateKey', setter, interpolate)
 */
function extractStateBinding(callExpr, timeline) {
  const args = callExpr.arguments;

  if (args.length >= 3) {
    const stateKeyArg = args[1];
    const setterArg = args[2];
    const interpolateArg = args[3];

    if (t.isStringLiteral(stateKeyArg) && t.isIdentifier(setterArg)) {
      const binding = {
        stateKey: stateKeyArg.value,
        setterName: setterArg.name,
        interpolate: false,
        stateType: 'unknown'
      };

      if (interpolateArg && t.isBooleanLiteral(interpolateArg)) {
        binding.interpolate = interpolateArg.value;
      }

      timeline.stateBindings.set(binding.stateKey, binding);
      console.log(`[Timeline] State binding: ${binding.stateKey} (interpolate: ${binding.interpolate})`);
    }
  }
}

/**
 * Extract keyframes from timeline.keyframes([...]) call
 */
function extractKeyframesArray(callExpr, timeline) {
  const arg = callExpr.arguments[0];

  if (t.isArrayExpression(arg)) {
    arg.elements.forEach(elem => {
      if (t.isObjectExpression(elem)) {
        const keyframe = parseKeyframeObject(elem);
        if (keyframe) {
          timeline.keyframes.push(keyframe);
          console.log(`[Timeline] Keyframe at ${keyframe.time}ms with state:`, Object.keys(keyframe.state).join(', '));
        }
      }
    });
  }
}

/**
 * Extract single keyframe from timeline.keyframe(time, state) call
 */
function extractSingleKeyframe(callExpr, timeline) {
  const args = callExpr.arguments;

  if (args.length >= 2) {
    const timeArg = args[0];
    const stateArg = args[1];

    if (t.isNumericLiteral(timeArg) && t.isObjectExpression(stateArg)) {
      const keyframe = parseKeyframeObject(stateArg);
      if (keyframe) {
        keyframe.time = timeArg.value;
        timeline.keyframes.push(keyframe);
        console.log(`[Timeline] Keyframe at ${keyframe.time}ms with state:`, Object.keys(keyframe.state).join(', '));
      }
    }
  }
}

/**
 * Parse keyframe object: { time: 0, state: { count: 0 }, label?: '', easing?: '' }
 */
function parseKeyframeObject(obj) {
  const keyframe = {
    time: 0,
    state: {},
    label: null,
    easing: null
  };

  obj.properties.forEach(prop => {
    if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
      const key = prop.key.name;
      const value = prop.value;

      switch (key) {
        case 'time':
          if (t.isNumericLiteral(value)) {
            keyframe.time = value.value;
          }
          break;
        case 'state':
          if (t.isObjectExpression(value)) {
            keyframe.state = parseStateObject(value);
          }
          break;
        case 'label':
          if (t.isStringLiteral(value)) {
            keyframe.label = value.value;
          }
          break;
        case 'easing':
          if (t.isStringLiteral(value)) {
            keyframe.easing = value.value;
          }
          break;
      }
    }
  });

  return keyframe;
}

/**
 * Parse state object from keyframe: { count: 0, color: 'blue' }
 */
function parseStateObject(obj) {
  const state = {};

  obj.properties.forEach(prop => {
    if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
      const key = prop.key.name;
      const value = prop.value;

      // Extract literal values
      if (t.isNumericLiteral(value)) {
        state[key] = value.value;
      } else if (t.isStringLiteral(value)) {
        state[key] = value.value;
      } else if (t.isBooleanLiteral(value)) {
        state[key] = value.value;
      } else if (t.isNullLiteral(value)) {
        state[key] = null;
      } else if (t.isUnaryExpression(value) && value.operator === '-' && t.isNumericLiteral(value.argument)) {
        // Handle negative numbers
        state[key] = -value.argument.value;
      }
    }
  });

  return state;
}

/**
 * Extract control method call (play, pause, stop, etc.)
 */
function extractControlMethod(callExpr, timeline, method) {
  timeline.controlMethods.push({
    method: method,
    arguments: callExpr.arguments.map(arg => {
      if (t.isNumericLiteral(arg)) return arg.value;
      if (t.isStringLiteral(arg)) return arg.value;
      if (t.isBooleanLiteral(arg)) return arg.value;
      return undefined;
    })
  });
}

/**
 * Validate timeline definition
 */
function validateTimeline(timeline) {
  const errors = [];

  // Check duration
  if (timeline.duration <= 0) {
    errors.push(`Timeline duration must be positive (got ${timeline.duration}ms)`);
  }

  // Check keyframe timing
  timeline.keyframes.forEach((kf, index) => {
    // Check time ordering
    if (index > 0) {
      const prevTime = timeline.keyframes[index - 1].time;
      if (kf.time <= prevTime) {
        errors.push(
          `Keyframe times must be ascending. ` +
          `Keyframe at index ${index} (${kf.time}ms) is not after previous (${prevTime}ms).`
        );
      }
    }

    // Check time within duration
    if (kf.time > timeline.duration) {
      errors.push(
        `Keyframe at ${kf.time}ms exceeds timeline duration (${timeline.duration}ms).`
      );
    }

    // Check state keys are bound
    Object.keys(kf.state).forEach(stateKey => {
      if (!timeline.stateBindings.has(stateKey)) {
        errors.push(
          `Keyframe at ${kf.time}ms references unbound state '${stateKey}'. ` +
          `Add useTimelineState(timeline, '${stateKey}', ...).`
        );
      }
    });
  });

  // Check state bindings exist in keyframes
  timeline.stateBindings.forEach((binding, stateKey) => {
    const usedInKeyframes = timeline.keyframes.some(kf => stateKey in kf.state);
    if (!usedInKeyframes) {
      console.warn(
        `[Timeline] State binding '${stateKey}' is not used in any keyframe`
      );
    }
  });

  return {
    valid: errors.length === 0,
    errors: errors
  };
}

module.exports = {
  analyzeTimeline
};
