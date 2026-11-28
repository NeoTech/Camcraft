//! Pane folds (#4483, UI plan P of #4245): a docked pane folds to its
//! title and gives the viewport the room. The fold flag is egui_dock's own
//! `LeafNode::collapsed`, so the dock's fold arrow, the saved panel setups
//! and this module all read one fact.
//! - Panes stacked top over bottom fold natively: egui_dock gives a folded
//!   leaf the tab-bar height.
//! - Panes side by side (the Browser at the left, Properties at the right)
//!   do not: egui_dock lays a horizontal split out by its fraction only. So
//!   a fold here stores the open share, holds the split at the kit's chip
//!   width on each frame (a window resize or a separator drag cannot widen
//!   it), and puts the share back on unfold. The shell draws the kit's
//!   title chip in the folded leaf; a click on it unfolds.
//! - The viewport never folds (the permanence law).
//! Folds are view state: immediate-mode, never a command, never in the
//! project file. Only the main area folds side panes: a float keeps
//! egui_dock's own behaviour.
//! Provenance: our own code, specced from the egui_dock 0.21.1 sources
//! (`Tree::node_update_collapsed`, `compute_rect_sizes`); no HSMWorks
//! material involved.

use std::cmp::max;

use camcraft_ui_kit::fold::{self as kit, FoldSide};
use egui_dock::{DockState, Node, NodeIndex, Tree};

use super::{BROWSER_FRAC, REOPEN_PROPS_FRAC};
use crate::egui_app::shell::Tab;
use crate::egui_app::theme::Tokens;

/// Each folded side pane, keyed by the first tab of its leaf: the open
/// split fraction to put back on unfold, and the split's width when the
/// fold law last held it (a change asks for one more frame).
pub(crate) type PaneFolds = Vec<(Tab, f32, f32)>;

/// The key marker of a saved setup that carries fold state (#4483). A
/// setup without it is from an older build, and loads with every pane
/// unfolded.
pub(crate) const FOLD_MARK: &str = "camcraft_pane_folds";

fn key(tree: &Tree<Tab>, i: NodeIndex) -> Option<Tab> {
    tree[i].get_leaf().and_then(|l| l.tabs.first().cloned())
}

/// The side-by-side split that holds leaf `i`, and which side it is on.
/// `None` for a leaf in a top-over-bottom split or at the root.
fn side_of(tree: &Tree<Tab>, i: NodeIndex) -> Option<(NodeIndex, FoldSide)> {
    let parent = i.parent()?;
    if !tree[parent].is_horizontal() {
        return None;
    }
    Some((parent, if i.is_left() { FoldSide::Left } else { FoldSide::Right }))
}

fn fallback(side: FoldSide) -> f32 {
    match side {
        FoldSide::Left => BROWSER_FRAC,
        FoldSide::Right => REOPEN_PROPS_FRAC,
    }
}

fn split_mut(tree: &mut Tree<Tab>, i: NodeIndex) -> Option<&mut egui_dock::SplitNode> {
    match &mut tree[i] {
        Node::Horizontal(s) | Node::Vertical(s) => Some(s),
        _ => None,
    }
}

/// The parents' fold counts after leaf `i` changed its flag. egui_dock's
/// own fix-up (`Tree::node_update_collapsed`) is crate-private; this is the
/// same walk, over the splits only.
pub(crate) fn fix_counts(tree: &mut Tree<Tab>, i: NodeIndex) {
    let mut up = i.parent();
    while let Some(p) = up {
        up = p.parent();
        let (l, r) = (p.left(), p.right());
        if r.0 >= tree.len() {
            break;
        }
        let (lc, rc) = (tree[l].collapsed_leaf_count(), tree[r].collapsed_leaf_count());
        let both = tree[l].is_collapsed() && tree[r].is_collapsed();
        let horizontal = tree[p].is_horizontal();
        if let Some(s) = split_mut(tree, p) {
            s.collapsed_leaf_count = if horizontal { max(lc, rc) } else { lc + rc };
            s.fully_collapsed = both;
        }
    }
}

/// The leaf indices of a tree, in index order.
fn leaves(tree: &Tree<Tab>) -> Vec<NodeIndex> {
    (0..tree.len()).map(NodeIndex).filter(|&i| tree[i].is_leaf()).collect()
}

fn find(tree: &Tree<Tab>, tab: &Tab) -> Option<NodeIndex> {
    tree.find_tab(tab).map(|(i, _)| i)
}

fn set_folded(tree: &mut Tree<Tab>, i: NodeIndex, folded: bool) {
    if let Some(leaf) = tree[i].get_leaf_mut() {
        leaf.collapsed = folded;
    }
    fix_counts(tree, i);
}

/// Fold the main-area leaf that holds `tab`, the way the dock's arrow does.
/// The viewport refuses. Returns whether the leaf is folded now. The shell
/// folds through the dock's arrow; this is the seam for tests and a later
/// fold command.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fold(dock: &mut DockState<Tab>, tab: &Tab) -> bool {
    let tree = dock.main_surface_mut();
    let Some(i) = find(tree, tab) else { return false };
    if tree[i].iter_tabs().any(|t| matches!(t, Tab::Viewport)) {
        return false;
    }
    set_folded(tree, i, true);
    true
}

/// Unfold the main-area leaf that holds `tab` and put its open share back.
pub(crate) fn unfold(dock: &mut DockState<Tab>, folds: &mut PaneFolds, tab: &Tab) {
    let tree = dock.main_surface_mut();
    let Some(i) = find(tree, tab) else { return };
    set_folded(tree, i, false);
    restore(tree, folds, i);
}

/// Put the stored open share of leaf `i` back, and forget it.
fn restore(tree: &mut Tree<Tab>, folds: &mut PaneFolds, i: NodeIndex) -> bool {
    let Some(k) = key(tree, i) else { return false };
    let Some(at) = folds.iter().position(|(t, _, _)| *t == k) else { return false };
    let (_, stored, _) = folds.remove(at);
    if let Some((p, side)) = side_of(tree, i)
        && let Some(s) = split_mut(tree, p)
    {
        s.fraction = kit::open_fraction(stored, fallback(side));
    }
    true
}

/// Whether the main-area leaf that holds `tab` is folded.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_folded(dock: &DockState<Tab>, tab: &Tab) -> bool {
    let tree = dock.main_surface();
    find(tree, tab).is_some_and(|i| tree[i].is_collapsed())
}

/// The fold law, once a frame before the dock lays out: the viewport is
/// unfolded, each folded side pane is held at chip width (its open share
/// stored first), and each pane unfolded by the dock's arrow gets its share
/// back. egui_dock clamps a split fraction to its separator minimum AFTER
/// the layout of each frame, so the hold is written again on every frame
/// and is not a change by itself. Returns whether a fold started or ended,
/// or a folded split changed width (a window resize), so the shell asks for
/// one more frame.
pub(crate) fn settle(dock: &mut DockState<Tab>, folds: &mut PaneFolds) -> bool {
    // A closed pane forgets its fold: a reopened one starts open.
    folds.retain(|(t, _, _)| dock.find_tab(t).is_some());
    let tree = dock.main_surface_mut();
    let mut moved = false;
    for i in leaves(tree) {
        if tree[i].is_collapsed() && tree[i].iter_tabs().any(|t| matches!(t, Tab::Viewport)) {
            set_folded(tree, i, false);
            moved = true;
            continue;
        }
        if !tree[i].is_collapsed() {
            moved |= restore(tree, folds, i);
            continue;
        }
        let Some((p, side)) = side_of(tree, i) else { continue };
        let Some(k) = key(tree, i) else { continue };
        let Some(s) = split_mut(tree, p) else { continue };
        let span = s.rect.width();
        match folds.iter_mut().find(|(t, _, _)| *t == k) {
            Some(entry) => {
                if (entry.2 - span).abs() >= 0.5 || entry.2.is_finite() != span.is_finite() {
                    entry.2 = span;
                    moved = true;
                }
            }
            None => {
                folds.push((k, s.fraction, span));
                moved = true;
            }
        }
        if let Some(target) = kit::chip_fraction(side, kit::CHIP_WIDTH, span) {
            s.fraction = target;
        }
    }
    moved
}

/// The folded side panes of the main area: the active tab and the leaf rect.
fn folded_side_panes(dock: &DockState<Tab>) -> Vec<(Tab, egui::Rect)> {
    let tree = dock.main_surface();
    leaves(tree)
        .into_iter()
        .filter(|&i| tree[i].is_collapsed() && side_of(tree, i).is_some())
        .filter_map(|i| {
            let leaf = tree[i].get_leaf()?;
            Some((leaf.tabs.get(leaf.active.0)?.clone(), leaf.rect))
        })
        .collect()
}

/// Draw the title chip of each folded side pane under its tab bar, after
/// the dock drew. A click on a chip unfolds that pane.
pub(crate) fn chips(ui: &mut egui::Ui, dock: &mut DockState<Tab>, folds: &mut PaneFolds, pal: &Tokens, tab_bar: f32) {
    let mut ask: Option<Tab> = None;
    for (tab, pane) in folded_side_panes(dock) {
        let Some(rect) = kit::chip_rect(pane, tab_bar) else { continue };
        let id = egui::Id::new(("pane-fold-chip", tab.title()));
        if kit::fold_chip(ui, rect, id, tab.title(), pal).clicked() {
            ask = Some(tab);
        }
    }
    if let Some(tab) = ask {
        unfold(dock, folds, &tab);
        ui.ctx().request_repaint();
    }
}

/// A saved copy of the dock holds each folded side pane's OPEN share, so
/// a load can unfold it to the width it had.
pub(crate) fn write_open_shares(copy: &mut DockState<Tab>, folds: &PaneFolds) {
    let tree = copy.main_surface_mut();
    for i in leaves(tree) {
        if !tree[i].is_collapsed() {
            continue;
        }
        let Some(k) = key(tree, i) else { continue };
        let Some(&(_, stored, _)) = folds.iter().find(|(t, _, _)| *t == k) else { continue };
        if let Some((p, side)) = side_of(tree, i)
            && let Some(s) = split_mut(tree, p)
        {
            s.fraction = kit::open_fraction(stored, fallback(side));
        }
    }
}

/// Unfold every pane of every surface: the load of a setup that an older
/// build saved, before panes could fold.
pub(crate) fn unfold_all(dock: &mut DockState<Tab>) {
    for (_, node) in dock.iter_all_nodes_mut() {
        match node {
            Node::Leaf(leaf) => leaf.collapsed = false,
            Node::Horizontal(s) | Node::Vertical(s) => {
                s.fully_collapsed = false;
                s.collapsed_leaf_count = 0;
            }
            Node::Empty => {}
        }
    }
}
