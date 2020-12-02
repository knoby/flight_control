// Things from relm
use relm::{connect, DrawHandler, Relm};
use relm_derive::Msg;

// GTK Imports
use gtk::prelude::*;
use gtk::DrawingArea;

#[derive(Debug)]
pub struct DataPoint {
    x: f64,
    y: f64,
}

struct DataSeries {
    data: Vec<DataPoint>,
    color: (f64, f64, f64),
    _label: String,
}

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    data: Vec<DataSeries>,
    min_max: Option<(f64, f64)>,
}

#[derive(Msg)]
pub enum Message {
    Draw,
    AddAngle(u32, f32, f32, f32),
    Clear,
}

pub struct Widget {
    model: Model,
    drawing_area: DrawingArea,
}

impl Widget {
    fn draw_background(&mut self, width: i32, height: i32) {
        let cx = self.model.draw_handler.get_context();

        cx.set_source_rgb(0.8, 0.8, 0.8);
        cx.paint();

        let w = width as f64 * 0.85;
        let h = height as f64 * 0.85;
        let x = width as f64 * 0.1;
        let y = height as f64 * 0.05;
        // Draw Background of the chart
        {
            // Black Background
            cx.rectangle(x, y, w, h);
            cx.set_source_rgb(0.0, 0.0, 0.0);
            cx.fill();
            // Devide in 10 parts in x and y direction
            for i in 0..12 {
                cx.set_line_width(1.0);
                cx.move_to(x + w / 11.0 * i as f64, y);
                cx.line_to(x + w / 11.0 * i as f64, y + h);
                cx.set_source_rgb(0.5, 0.5, 0.5);
                cx.stroke();
                cx.move_to(x, y + h / 11.0 * i as f64);
                cx.line_to(x + w, y + h / 11.0 * i as f64);
                cx.set_source_rgb(0.5, 0.5, 0.5);
                cx.stroke();
            }
        }
        // Draw the data
        {
            // Find x min, x max, y min, y max
            let mut x_min;
            let mut y_min;
            let mut x_max;
            let mut y_max;
            if let Some((min, max)) = self.model.min_max {
                x_min = std::f64::MAX;
                x_max = std::f64::MIN;
                y_min = min;
                y_max = max;
                for series in self.model.data.iter() {
                    for point in series.data.iter() {
                        x_min = x_min.min(point.x);
                        x_max = x_max.max(point.x);
                    }
                }
            } else {
                x_min = std::f64::MAX;
                x_max = std::f64::MIN;
                y_min = std::f64::MAX;
                y_max = std::f64::MIN;
                for series in self.model.data.iter() {
                    for point in series.data.iter() {
                        x_min = x_min.min(point.x);
                        x_max = x_max.max(point.x);
                        y_min = y_min.min(point.y);
                        y_max = y_max.max(point.y);
                    }
                }
            }
            // Check if min==max
            if (x_max - x_min).abs() <= f64::EPSILON {
                x_max += 0.5;
                x_min -= 0.5;
            }
            if (y_max - y_min).abs() <= f64::EPSILON {
                y_max += 0.5;
                y_min -= 0.5;
            }
            // Calculate scaling
            let x_offset = -x_min;
            let x_scaling = w / (x_max - x_min);
            let y_offset = -y_min;
            let y_scaling = h / (y_max - y_min);

            // Draw the data
            for series in self.model.data.iter() {
                cx.set_line_width(2.0);
                cx.set_source_rgb(series.color.0, series.color.1, series.color.2);
                if !series.data.is_empty() {
                    cx.move_to(
                        x + (x_offset + series.data[0].x) * x_scaling,
                        y + h - (y_offset + series.data[0].y.min(y_max).max(y_min)) * y_scaling,
                    );
                }
                for point in series.data.iter().skip(1) {
                    cx.line_to(
                        x + (x_offset + point.x) * x_scaling,
                        y + h - (y_offset + point.y.min(y_max).max(y_min)) * y_scaling,
                    );
                }
                cx.stroke();
            }
        }
    }
}

impl relm::Update for Widget {
    type Model = Model;
    type ModelParam = ();
    type Msg = Message;

    fn model(_relm: &Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        let draw_handler = DrawHandler::new().unwrap();

        // Create some sample data
        let mut data: Vec<DataSeries> = Vec::new();

        data.push(DataSeries {
            data: Vec::new(),
            color: (1.0, 0.0, 0.0),
            _label: "Roll [°]".to_string(),
        });
        data.push(DataSeries {
            data: Vec::new(),
            color: (0.0, 1.0, 0.0),
            _label: "Pitch [°]".to_string(),
        });
        data.push(DataSeries {
            data: Vec::new(),
            color: (0.0, 0.0, 1.0),
            _label: "Yaw[°]".to_string(),
        });

        Self::Model {
            draw_handler,
            data,
            min_max: None,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Message::Draw => {
                let allocation = self.drawing_area.get_allocation();
                let width = allocation.width;
                let height = allocation.height;
                self.draw_background(width, height)
            }
            Message::Clear => {
                for data in self.model.data.iter_mut() {
                    data.data.clear();
                }
            }
            Message::AddAngle(time, roll, pitch, yaw) => {
                self.model.data[0].data.push(DataPoint {
                    x: time as f64,
                    y: roll as f64,
                });
                self.model.data[1].data.push(DataPoint {
                    x: time as f64,
                    y: pitch as f64,
                });
                self.model.data[2].data.push(DataPoint {
                    x: time as f64,
                    y: yaw as f64,
                });
                if self.model.data[0].data.len() >= 200 {
                    for data in self.model.data.iter_mut() {
                        data.data.remove(0);
                    }
                }
            }
        }
    }
}

impl relm::Widget for Widget {
    type Root = DrawingArea;

    fn root(&self) -> Self::Root {
        self.drawing_area.clone()
    }

    fn view(relm: &Relm<Self>, mut model: Self::Model) -> Self {
        // Create the drawing area
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(400, 300);
        model.draw_handler.init(&drawing_area);

        // Connect the draw event
        connect!(
            relm,
            drawing_area,
            connect_draw(_, _),
            return (Some(Message::Draw), Inhibit(false))
        );

        Self {
            drawing_area,
            model,
        }
    }
}
