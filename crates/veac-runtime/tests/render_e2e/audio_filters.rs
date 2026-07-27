use tempfile::tempdir;

use super::support::*;

#[test]
fn eq_high_pass_and_low_pass_change_measured_frequency_energy() {
    let temp = tempdir().unwrap();
    let source = dual_tone_wav(temp.path(), "dual", 200, 3_000);
    let cases = [
        (
            "highpass",
            AudioProcessor::HighPass {
                frequency_hz: 1_000.0,
                q: 0.707,
                poles: 2,
            },
            3_000.0,
            200.0,
        ),
        (
            "lowpass",
            AudioProcessor::LowPass {
                frequency_hz: 1_000.0,
                q: 0.707,
                poles: 2,
            },
            200.0,
            3_000.0,
        ),
    ];
    for (name, processor, kept, removed) in cases {
        let output = temp.path().join(format!("{name}.mp4"));
        render_audio_chain(&source, 1_000, vec![processor], None, &output);
        let samples = audio_samples(&output, 0.2, 0.5);
        let kept = tone_power(&samples, kept);
        let removed = tone_power(&samples, removed);
        assert!(
            kept > removed * 6.0,
            "{name}: kept={kept}, removed={removed}"
        );
    }

    let eq_source = dual_tone_wav(temp.path(), "eq-dual", 1_000, 3_000);
    let eq_output = temp.path().join("eq.mp4");
    render_audio_chain(
        &eq_source,
        1_000,
        vec![AudioProcessor::ParametricEq {
            bands: vec![ParametricEqBand {
                frequency_hz: 1_000.0,
                gain_db: -18.0,
                q: 4.0,
            }],
        }],
        None,
        &eq_output,
    );
    let samples = audio_samples(&eq_output, 0.2, 0.5);
    let removed = tone_power(&samples, 1_000.0);
    let kept = tone_power(&samples, 3_000.0);
    assert!(kept > removed * 4.0, "EQ kept={kept}, removed={removed}");
}
