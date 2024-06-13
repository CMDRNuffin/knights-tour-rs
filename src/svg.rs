use std::{io::{Result, Write}, time::Duration};

use crate::move_graph::{MoveGraph, NodesIterator};
use svg_macro::svg;

pub fn render_svg(writer: &mut impl Write, move_graph: &MoveGraph, duration: Duration) -> Result<()> {
    const MARGIN: usize = 10;
    const TITLE_BAR: usize = 20;
    const END_BORDER: usize = 1;
    let width = move_graph.width() as usize * 10 + END_BORDER;
    let file_width = (width + 2 * MARGIN).max(250);
    let height = move_graph.height() as usize * 10 + END_BORDER;
    let file_height = height + MARGIN + TITLE_BAR;
    let moves_iter = ConnectionsIter::new(move_graph, TITLE_BAR, MARGIN);
    let duration = format!("💩 Elapsed time: {}.{:03} seconds 💩", duration.as_secs(), duration.subsec_millis());
    svg! { writer =>
        <svg xmlns="http://www.w3.org/2000/svg" width=#file_width height=#file_height>
            <defs>
                <pattern id="grid" width="10" height="10" patternUnits="userSpaceOnUse">
                    // grid pattern (1px left and top line on a 10*10 square)
                    <path d="M 10 0 L 0 0 0 10" fill="none" stroke="gray" stroke-width="1" />
                </pattern>
            </defs>
            <text x=#MARGIN y=#MARGIN font-size="15" dominant-baseline="middle" font-family="Arial" fill="black">#duration</text>
            <rect x=#MARGIN y=#TITLE_BAR #width #height fill="url(#grid)" />
            #(#moves_iter)*
        </svg>
    };

    Ok(())
}

struct ConnectionsIter<'a> {
    iter: NodesIterator<'a>,
    v_offset: usize,
    h_offset: usize,
}

impl<'a> ConnectionsIter<'a> {
    fn new(graph: &'a MoveGraph<'a>, v_offset: usize, h_offset: usize) -> Self {
        ConnectionsIter{ iter: graph.nodes(), v_offset, h_offset }
    }
}

impl<'a> Iterator for ConnectionsIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.iter.next()?;
        if let Some(next) = node.next() {
            let pos = (node.pos().col() as usize * 10 + 5 + self.h_offset, node.pos().row() as usize * 10 + 5 + self.v_offset);
            let next = (next.col() as usize * 10 + 5 + self.h_offset, next.row() as usize * 10 + 5 + self.v_offset);
            let res = format!("<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"1.5\" />", pos.0, pos.1, next.0, next.1);
            Some(res)
        }
        else{
            self.next()
        }
    }
}