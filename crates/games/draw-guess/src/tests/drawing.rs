use crate::{
    CANVAS_WIDTH, CanvasDocument, CanvasPoint, DrawAction, DrawGuessError, Ink, RasterCanvas,
    Stroke,
};

fn point(x: u16, y: u16) -> CanvasPoint {
    CanvasPoint { x, y }
}

fn stroke(document: &mut CanvasDocument, x: u16, color: Ink) {
    document
        .apply(&DrawAction::Begin {
            point: point(x, 40),
            color,
            size: 8,
            eraser: false,
        })
        .unwrap();
    document
        .apply(&DrawAction::Extend {
            points: vec![point(x, 80)],
        })
        .unwrap();
    document.apply(&DrawAction::End).unwrap();
}

fn pixel(raster: &RasterCanvas, x: u16, y: u16) -> &[u8] {
    let index = (usize::from(y) * usize::from(CANVAS_WIDTH) + usize::from(x)) * 4;
    &raster.pixels[index..index + 3]
}

#[test]
fn streaming_rasterization_matches_a_complete_stroke_pixel_for_pixel() {
    let mut stroke = Stroke {
        id: 1,
        color: Ink([20, 100, 180]),
        size: 7,
        eraser: false,
        points: vec![
            point(10, 10),
            point(10, 20),
            point(35, 40),
            point(70, 30),
            point(83, 97),
        ],
    };
    let mut complete = RasterCanvas::default();
    complete.stroke(&stroke);
    let points = stroke.points.clone();
    let mut streamed = RasterCanvas::default();
    stroke.points.clear();
    for point in points {
        let before = stroke.points.len();
        stroke.points.push(point);
        streamed.append_stroke(&stroke, before);
    }
    assert_eq!(streamed.pixels, complete.pixels);
    assert_eq!(streamed.stroke_at(point(70, 30)), Some(1));
}

#[test]
fn stroke_erasure_undo_redo_and_branching_restore_the_exact_document() {
    let mut document = CanvasDocument::default();
    stroke(&mut document, 40, Ink::BLACK);
    let mut raster = RasterCanvas::default();
    raster.render(&document);
    let original = raster.pixels.clone();
    let id = raster.stroke_at(point(40, 50)).unwrap();
    document.apply(&DrawAction::EraseStroke { id }).unwrap();
    raster.render(&document);
    assert_eq!(pixel(&raster, 40, 50), &[255; 3]);
    document.apply(&DrawAction::Undo).unwrap();
    raster.render(&document);
    assert_eq!(raster.pixels, original);
    document.apply(&DrawAction::Redo).unwrap();
    raster.render(&document);
    assert_eq!(pixel(&raster, 40, 50), &[255; 3]);
    document.apply(&DrawAction::Undo).unwrap();
    stroke(&mut document, 80, Ink([255, 0, 0]));
    assert!(!document.can_redo());
    assert_eq!(
        document.apply(&DrawAction::Redo),
        Err(DrawGuessError::NothingToRedo)
    );
}

#[test]
fn area_eraser_and_clear_are_individually_reversible() {
    let mut document = CanvasDocument::default();
    stroke(&mut document, 40, Ink::BLACK);
    document
        .apply(&DrawAction::Begin {
            point: point(40, 60),
            color: Ink::BLACK,
            size: 16,
            eraser: true,
        })
        .unwrap();
    document.apply(&DrawAction::End).unwrap();
    let mut raster = RasterCanvas::default();
    raster.render(&document);
    assert_eq!(pixel(&raster, 40, 60), &[255; 3]);
    assert_eq!(pixel(&raster, 40, 40), &Ink::BLACK.0);
    document.apply(&DrawAction::Clear).unwrap();
    raster.render(&document);
    assert_eq!(pixel(&raster, 40, 40), &[255; 3]);
    document.apply(&DrawAction::Undo).unwrap();
    raster.render(&document);
    assert_eq!(pixel(&raster, 40, 40), &Ink::BLACK.0);
    document.apply(&DrawAction::Undo).unwrap();
    raster.render(&document);
    assert_eq!(pixel(&raster, 40, 60), &Ink::BLACK.0);
}

#[test]
fn flood_fill_stays_inside_a_closed_outline_and_can_be_undone() {
    let mut document = CanvasDocument::default();
    document
        .apply(&DrawAction::Begin {
            point: point(30, 30),
            color: Ink::BLACK,
            size: 6,
            eraser: false,
        })
        .unwrap();
    document
        .apply(&DrawAction::Extend {
            points: vec![point(90, 30), point(90, 90), point(30, 90), point(30, 30)],
        })
        .unwrap();
    document.apply(&DrawAction::End).unwrap();
    let red = Ink([240, 30, 30]);
    document
        .apply(&DrawAction::Fill {
            point: point(60, 60),
            color: red,
        })
        .unwrap();
    let mut raster = RasterCanvas::default();
    raster.render(&document);
    assert_eq!(pixel(&raster, 60, 60), &red.0);
    assert_eq!(pixel(&raster, 120, 60), &[255; 3]);
    document.apply(&DrawAction::Undo).unwrap();
    raster.render(&document);
    assert_eq!(pixel(&raster, 60, 60), &[255; 3]);
}

#[test]
fn invalid_batches_are_atomic_and_live_strokes_support_undo() {
    let mut document = CanvasDocument::default();
    document
        .apply(&DrawAction::Begin {
            point: point(10, 10),
            color: Ink::BLACK,
            size: 4,
            eraser: false,
        })
        .unwrap();
    let before = document.clone();
    assert_eq!(
        document.apply(&DrawAction::Extend {
            points: vec![point(11, 10), point(960, 10)]
        }),
        Err(DrawGuessError::InvalidDrawing)
    );
    assert_eq!(document, before);
    document.apply(&DrawAction::Undo).unwrap();
    assert!(document.operations().is_empty());
    assert!(document.pending().is_none());
    document.apply(&DrawAction::Redo).unwrap();
    assert_eq!(document.operations().len(), 1);
}
