/**
 * Timeline Generator Module
 *
 * Generates C# attributes and timeline metadata files from timeline analysis
 */

/**
 * Generate C# attributes for timeline
 *
 * @param {Object} timeline - Timeline metadata from analyzer
 * @returns {string[]} - Array of C# attribute strings
 */
function generateTimelineAttributes(timeline) {
  const attributes = [];

  // 1. Generate [Timeline] attribute
  const timelineAttrParts = [
    `[Timeline("${timeline.timelineId}", ${timeline.duration}`
  ];

  if (timeline.repeat) {
    timelineAttrParts.push(', Repeat = true');
  }

  if (timeline.repeatCount && timeline.repeatCount !== -1) {
    timelineAttrParts.push(`, RepeatCount = ${timeline.repeatCount}`);
  }

  if (timeline.easing && timeline.easing !== 'linear') {
    timelineAttrParts.push(`, Easing = "${timeline.easing}"`);
  }

  timelineAttrParts.push(')]');
  attributes.push(timelineAttrParts.join(''));

  // 2. Generate [TimelineKeyframe] attributes
  timeline.keyframes.forEach(kf => {
    Object.entries(kf.state).forEach(([stateKey, value]) => {
      const valueStr = formatCSharpValue(value);
      let keyframeAttr = `[TimelineKeyframe(${kf.time}, "${stateKey}", ${valueStr}`;

      if (kf.label) {
        keyframeAttr += `, Label = "${kf.label}"`;
      }

      if (kf.easing) {
        keyframeAttr += `, Easing = "${kf.easing}"`;
      }

      keyframeAttr += ')]';
      attributes.push(keyframeAttr);
    });
  });

  // 3. Generate [TimelineStateBinding] attributes
  timeline.stateBindings.forEach((binding, stateKey) => {
    let bindingAttr = `[TimelineStateBinding("${stateKey}"`;

    if (binding.interpolate) {
      bindingAttr += ', Interpolate = true';
    }

    bindingAttr += ')]';
    attributes.push(bindingAttr);
  });

  return attributes;
}

/**
 * Format a value for C# code
 */
function formatCSharpValue(value) {
  if (typeof value === 'string') {
    // Escape quotes and backslashes
    const escaped = value.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
    return `"${escaped}"`;
  } else if (typeof value === 'number') {
    return value.toString();
  } else if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  } else if (value === null) {
    return 'null';
  }
  return 'null';
}

/**
 * Generate timeline metadata JSON file
 *
 * @param {string} componentName - Component name
 * @param {Object} timeline - Timeline metadata
 * @param {Object} templates - Existing template metadata
 * @returns {Object} - Timeline metadata JSON
 */
function generateTimelineMetadataFile(componentName, timeline, templates) {
  return {
    component: componentName,
    timelineId: timeline.timelineId,
    duration: timeline.duration,
    repeat: timeline.repeat || false,
    repeatCount: timeline.repeatCount || -1,
    easing: timeline.easing || 'linear',
    stateBindings: Object.fromEntries(
      Array.from(timeline.stateBindings.entries()).map(([key, binding]) => [
        key,
        {
          interpolate: binding.interpolate,
          type: binding.stateType,
          setterName: binding.setterName
        }
      ])
    ),
    keyframes: timeline.keyframes.map(kf => ({
      time: kf.time,
      label: kf.label,
      state: kf.state,
      easing: kf.easing,
      affectedPaths: extractAffectedPaths(kf.state, templates)
    })),
    generatedAt: Date.now()
  };
}

/**
 * Extract hex paths affected by state changes in keyframe
 *
 * @param {Object} state - State object from keyframe
 * @param {Object} templates - Template metadata
 * @returns {string[]} - Array of affected hex paths
 */
function extractAffectedPaths(state, templates) {
  const paths = new Set();

  if (!templates) {
    return [];
  }

  // Find templates that reference these state keys
  Object.entries(templates).forEach(([path, template]) => {
    if (template.bindings && Array.isArray(template.bindings)) {
      // Check if any of the bindings match state keys
      template.bindings.forEach(binding => {
        // Binding might be "count" or "item.count" or nested
        const baseKey = binding.split('.')[0];
        if (baseKey in state) {
          paths.add(path);
        }
      });
    }
  });

  return Array.from(paths).sort();
}

module.exports = {
  generateTimelineAttributes,
  generateTimelineMetadataFile
};
