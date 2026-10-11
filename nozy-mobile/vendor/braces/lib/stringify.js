'use strict';

const utils = require('./utils');
const { MAX_DEPTH } = require('./constants');

module.exports = (ast, options = {}) => {
  const stringify = (node, parent = {}, depth = 0) => {
    if (depth > MAX_DEPTH) {
      throw new SyntaxError(`brace expansion depth exceeds max depth (${MAX_DEPTH})`);
    }
    const invalidBlock = options.escapeInvalid && utils.isInvalidBrace(parent);
    const invalidNode = node.invalid === true && options.escapeInvalid === true;
    let output = '';

    if (node.value) {
      if ((invalidBlock || invalidNode) && utils.isOpenOrClose(node)) {
        return '\\' + node.value;
      }
      return node.value;
    }

    if (node.value) {
      return node.value;
    }

    if (node.nodes) {
      const next = node.type === 'brace' || node.type === 'paren' ? depth + 1 : depth;
      for (const child of node.nodes) {
        output += stringify(child, node, next);
      }
    }
    return output;
  };

  return stringify(ast);
};

