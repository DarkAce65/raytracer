use super::BOX_BLUR_ITERATIONS;

#[allow(dead_code)]
pub fn repeated_box_blur(input: &[f64], width: usize, radius: u16) -> Vec<f64> {
    let mut output = box_blur(&input, width, radius);

    for _ in 1..BOX_BLUR_ITERATIONS {
        output = box_blur(&output, width, radius);
    }

    output
}

fn box_blur(input: &[f64], width: usize, radius: u16) -> Vec<f64> {
    let radius = radius.into();

    vertical_1d_blur_pass(
        &horizontal_1d_blur_pass(&input, width, radius),
        width,
        radius,
    )
}

fn vertical_1d_blur_pass(input: &[f64], width: usize, radius: usize) -> Vec<f64> {
    let scale = 1.0 / (radius as f64 * 2.0 + 1.0);
    let mut output = vec![0.0; input.len()];

    let height = input.len() / width;

    for col_index in 0..width {
        let mut blur_acc = input[col_index];
        for index in 0..radius {
            blur_acc += input[col_index] + input[col_index + index.min(height - 1) * width];
        }

        for index in 0..height {
            blur_acc += input[col_index + (index + radius).min(height - 1) * width]
                - input[col_index + index.saturating_sub(radius + 1) * width];

            output[col_index + index * width] = scale * blur_acc;
        }
    }

    output
}

fn horizontal_1d_blur_pass(input: &[f64], width: usize, radius: usize) -> Vec<f64> {
    let scale = 1.0 / (radius as f64 * 2.0 + 1.0);
    let mut output = vec![0.0; input.len()];

    for row in 0..(input.len() / width) {
        let row_index = row * width;

        let mut blur_acc = input[row_index];
        for index in 0..radius {
            blur_acc += input[row_index] + input[row_index + index.min(width - 1)];
        }

        for index in 0..width {
            blur_acc += input[row_index + (index + radius).min(width - 1)]
                - input[row_index + index.saturating_sub(radius + 1)];

            output[row_index + index] = scale * blur_acc;
        }
    }

    output
}
