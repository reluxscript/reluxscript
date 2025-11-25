/**
 * Hex Path Generator for Minimact
 *
 * Generates lexicographically sortable, insertion-friendly paths using 8-digit hex codes.
 *
 * Benefits:
 * - No renumbering needed when inserting elements
 * - String comparison works for sorting
 * - Billions of slots between any two elements
 * - Easy to visualize tree structure
 *
 * Example:
 *   div [10000000]
 *     span [10000000.10000000]
 *     span [10000000.20000000]
 *     p [10000000.30000000]
 *   section [20000000]
 */

class HexPathGenerator {
  /**
   * @param {number} gap - Spacing between elements (default: 0x10000000 = 268,435,456)
   */
  constructor(gap = 0x10000000) {
    this.gap = gap;
    this.counters = {}; // Track counters per parent path
  }

  /**
   * Generate next hex code for a given parent path
   * @param {string} parentPath - Parent path (e.g., "10000000" or "10000000.1")
   * @returns {string} - Next hex code (compact: 1, 2, 3...a, b, c...10, 11...)
   */
  next(parentPath = '') {
    if (!this.counters[parentPath]) {
      this.counters[parentPath] = 0;
    }

    this.counters[parentPath]++;
    // For root level (empty parent), use gap-based spacing for components
    // For child elements, use simple sequential hex (1, 2, 3...a, b, c...)
    const hexValue = (parentPath === '' ? this.counters[parentPath] * this.gap : this.counters[parentPath]).toString(16);
    // Truncate trailing zeroes to keep paths compact (1 instead of 10000000)
    return hexValue.replace(/0+$/, '') || '0';
  }

  /**
   * Build full path by joining parent and child
   * @param {string} parentPath - Parent path
   * @param {string} childHex - Child hex code
   * @returns {string} - Full path (e.g., "10000000.20000000")
   */
  buildPath(parentPath, childHex) {
    return parentPath ? `${parentPath}.${childHex}` : childHex;
  }

  /**
   * Parse path into segments
   * @param {string} path - Full path (e.g., "10000000.20000000.30000000")
   * @returns {string[]} - Array of hex segments
   */
  parsePath(path) {
    return path.split('.');
  }

  /**
   * Get depth of a path (number of segments)
   * @param {string} path - Full path
   * @returns {number} - Depth (0 for root, 1 for first level, etc.)
   */
  getDepth(path) {
    return path ? this.parsePath(path).length : 0;
  }

  /**
   * Get parent path
   * @param {string} path - Full path
   * @returns {string|null} - Parent path or null if root
   */
  getParentPath(path) {
    const lastDot = path.lastIndexOf('.');
    return lastDot > 0 ? path.substring(0, lastDot) : null;
  }

  /**
   * Check if path1 is ancestor of path2
   * @param {string} ancestorPath - Potential ancestor
   * @param {string} descendantPath - Potential descendant
   * @returns {boolean}
   */
  isAncestorOf(ancestorPath, descendantPath) {
    return descendantPath.startsWith(ancestorPath + '.');
  }

  /**
   * Reset counter for a specific parent (useful for testing)
   * @param {string} parentPath - Parent path to reset
   */
  reset(parentPath = '') {
    delete this.counters[parentPath];
  }

  /**
   * Reset all counters (useful for testing)
   */
  resetAll() {
    this.counters = {};
  }

  /**
   * Generate a path between two existing paths (for future insertion)
   * @param {string} path1 - First path
   * @param {string} path2 - Second path
   * @returns {string} - Midpoint path
   */
  static generatePathBetween(path1, path2) {
    const segments1 = path1.split('.');
    const segments2 = path2.split('.');

    // Find common prefix length
    let commonLength = 0;
    while (commonLength < Math.min(segments1.length, segments2.length) &&
           segments1[commonLength] === segments2[commonLength]) {
      commonLength++;
    }

    // Get the differing segments
    const seg1 = commonLength < segments1.length
      ? parseInt(segments1[commonLength], 16)
      : 0;
    const seg2 = commonLength < segments2.length
      ? parseInt(segments2[commonLength], 16)
      : 0;

    // Generate midpoint
    const midpoint = Math.floor((seg1 + seg2) / 2);
    const newSegment = midpoint.toString(16).padStart(8, '0');

    // Build new path
    const prefix = segments1.slice(0, commonLength).join('.');
    return prefix ? `${prefix}.${newSegment}` : newSegment;
  }

  /**
   * Check if there's sufficient gap between two paths
   * @param {string} path1 - First path
   * @param {string} path2 - Second path
   * @param {number} minGap - Minimum required gap (default: 0x00100000)
   * @returns {boolean}
   */
  static hasSufficientGap(path1, path2, minGap = 0x00100000) {
    const seg1 = parseInt(path1.split('.').pop(), 16);
    const seg2 = parseInt(path2.split('.').pop(), 16);
    return Math.abs(seg2 - seg1) > minGap;
  }
}

module.exports = { HexPathGenerator };
