use crate::colors::ColorScheme;
use crate::objects::{DrawCtx, ID};
use crate::render::{RenderOptions, Renderable, PARCEL_BOUNDARY_THICKNESS};
use ezgui::{Color, GfxCtx};
use geom::{PolyLine, Polygon};
use map_model::{Map, Parcel, ParcelID};

const COLORS: [Color; 14] = [
    // TODO these are awful choices
    Color::rgba_f(1.0, 1.0, 0.0, 1.0),
    Color::rgba_f(1.0, 0.0, 1.0, 1.0),
    Color::rgba_f(0.0, 1.0, 1.0, 1.0),
    Color::rgba_f(0.5, 0.2, 0.7, 1.0),
    Color::rgba_f(0.5, 0.5, 0.0, 0.5),
    Color::rgba_f(0.5, 0.0, 0.5, 0.5),
    Color::rgba_f(0.0, 0.5, 0.5, 0.5),
    Color::rgba_f(0.0, 0.0, 0.5, 0.5),
    Color::rgba_f(0.3, 0.2, 0.5, 0.5),
    Color::rgba_f(0.4, 0.2, 0.5, 0.5),
    Color::rgba_f(0.5, 0.2, 0.5, 0.5),
    Color::rgba_f(0.6, 0.2, 0.5, 0.5),
    Color::rgba_f(0.7, 0.2, 0.5, 0.5),
    Color::rgba_f(0.8, 0.2, 0.5, 0.5),
];

pub struct DrawParcel {
    pub id: ParcelID,
    // TODO could maybe get away with not storing these at all, since we can't select them. might
    // totally get rid of parcels, since use case is low... keep for now
    boundary_polygon: Polygon,
    pub fill_polygon: Polygon,
}

impl DrawParcel {
    pub fn new(p: &Parcel, cs: &ColorScheme) -> (DrawParcel, Vec<(Color, Polygon)>) {
        let boundary_polygon =
            PolyLine::make_polygons_for_boundary(p.points.clone(), PARCEL_BOUNDARY_THICKNESS);
        let fill_polygon = Polygon::new(&p.points);
        let default_draw = vec![
            (COLORS[p.block % COLORS.len()], fill_polygon.clone()),
            (
                cs.get_def("parcel boundary", Color::grey(0.3)),
                boundary_polygon.clone(),
            ),
        ];

        (
            DrawParcel {
                id: p.id,
                boundary_polygon,
                fill_polygon,
            },
            default_draw,
        )
    }
}

impl Renderable for DrawParcel {
    fn get_id(&self) -> ID {
        ID::Parcel(self.id)
    }

    fn draw(&self, g: &mut GfxCtx, opts: RenderOptions, ctx: &DrawCtx) {
        if let Some(color) = opts.color {
            g.draw_polygon_batch(vec![
                (color, &self.fill_polygon),
                (ctx.cs.get("parcel boundary"), &self.boundary_polygon),
            ]);
        }
    }

    fn get_outline(&self, _: &Map) -> Polygon {
        self.fill_polygon.clone()
    }
}
