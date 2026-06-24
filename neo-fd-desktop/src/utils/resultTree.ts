import type { ScanResultItem } from '../stores/scan';

export interface TreeNode {
  rowKind: 'tree';
  id: string;
  name: string;
  type: 'folder' | 'file' | 'match';
  depth: number;
  isOpen: boolean;
  parentId: string | null;
  children: string[];
  matchCount: number;
  scanResult?: ScanResultItem;
}

function normalizedPath(path: string): string {
  return path.replace(/\\/g, '/');
}

function getCommonDirectory(paths: string[]): string {
  if (paths.length === 0) return '';
  if (paths.length === 1) {
    const parts = paths[0].split('/');
    parts.pop();
    return parts.join('/');
  }

  const splitPaths = paths.map((path) => path.split('/'));
  const common: string[] = [];
  const firstPathParts = splitPaths[0];

  for (let i = 0; i < firstPathParts.length - 1; i += 1) {
    const part = firstPathParts[i];
    const isCommon = splitPaths.every((parts) => parts[i] === part);
    if (!isCommon) break;
    common.push(part);
  }

  return common.join('/');
}

function typeSortValue(type: TreeNode['type']): number {
  if (type === 'folder') return 0;
  if (type === 'file') return 1;
  return 2;
}

function sortTreeNodes(a: TreeNode, b: TreeNode): number {
  const typeDiff = typeSortValue(a.type) - typeSortValue(b.type);
  if (typeDiff !== 0) return typeDiff;

  if (a.type === 'match' && b.type === 'match') {
    const aLine = a.scanResult?.line_num ?? 0;
    const bLine = b.scanResult?.line_num ?? 0;
    if (aLine !== bLine) return aLine - bLine;

    const aId = a.scanResult?.id ?? 0;
    const bId = b.scanResult?.id ?? 0;
    return aId - bId;
  }

  return a.name.localeCompare(b.name);
}

export function buildResultTreeRows(
  results: readonly ScanResultItem[],
  collapsedIds: ReadonlySet<string>,
): TreeNode[] {
  if (results.length === 0) return [];

  const paths = results.map((result) => normalizedPath(result.path));
  const commonPrefix = getCommonDirectory(paths);
  const nodesMap = new Map<string, TreeNode>();
  const root: TreeNode = {
    rowKind: 'tree',
    id: 'root',
    name: 'Root',
    type: 'folder',
    depth: -1,
    isOpen: true,
    parentId: null,
    children: [],
    matchCount: 0,
  };
  nodesMap.set(root.id, root);

  for (const result of results) {
    const path = normalizedPath(result.path);
    let relativePath = path;
    if (commonPrefix && path.startsWith(commonPrefix)) {
      relativePath = path.slice(commonPrefix.length);
      if (relativePath.startsWith('/')) {
        relativePath = relativePath.slice(1);
      }
    }

    const pathParts = relativePath.split('/').filter(Boolean);
    let currentParentId = root.id;
    let accumulatedPath = commonPrefix;

    pathParts.forEach((part, partIndex) => {
      const isLast = partIndex === pathParts.length - 1;
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${part}` : part;
      const nodeId = `path:${accumulatedPath}`;

      if (!nodesMap.has(nodeId)) {
        const parentNode = nodesMap.get(currentParentId);
        if (parentNode) {
          parentNode.children.push(nodeId);
        }

        nodesMap.set(nodeId, {
          rowKind: 'tree',
          id: nodeId,
          name: part,
          type: isLast ? 'file' : 'folder',
          depth: partIndex,
          isOpen: !collapsedIds.has(nodeId),
          parentId: currentParentId,
          children: [],
          matchCount: 0,
        });
      }

      const node = nodesMap.get(nodeId);
      if (node) {
        node.matchCount += 1;
      }
      currentParentId = nodeId;
    });

    const fileNode = nodesMap.get(currentParentId);
    if (fileNode) {
      const matchNodeId = `match:${result.id}`;
      nodesMap.set(matchNodeId, {
        rowKind: 'tree',
        id: matchNodeId,
        name: `行 ${result.line_num}: [${result.pattern_name}] ${result.matched_text}`,
        type: 'match',
        depth: fileNode.depth + 1,
        isOpen: !collapsedIds.has(matchNodeId),
        parentId: currentParentId,
        children: [],
        matchCount: 1,
        scanResult: result,
      });
      fileNode.children.push(matchNodeId);
    }

    root.matchCount += 1;
  }

  const flattened: TreeNode[] = [];
  function traverse(nodeId: string) {
    const node = nodesMap.get(nodeId);
    if (!node) return;

    if (nodeId !== root.id) {
      flattened.push(node);
    }

    if (!node.isOpen) return;

    const childNodes = node.children
      .map((id) => nodesMap.get(id))
      .filter((child): child is TreeNode => Boolean(child))
      .sort(sortTreeNodes);

    for (const child of childNodes) {
      traverse(child.id);
    }
  }

  traverse(root.id);
  return flattened;
}
