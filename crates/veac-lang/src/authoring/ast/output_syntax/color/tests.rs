use super::*;

#[test]
fn every_authored_color_token_round_trips_through_the_canonical_model() {
    for &authored in ColorPrimaries::ALL {
        let canonical = CanonicalPrimaries::from(authored);
        assert_eq!(ColorPrimaries::from(canonical), authored);
    }
    for &authored in ColorTransfer::ALL {
        let canonical = CanonicalTransfer::from(authored);
        assert_eq!(ColorTransfer::from(canonical), authored);
    }
    for &authored in ColorMatrix::ALL {
        let canonical = CanonicalMatrix::from(authored);
        assert_eq!(ColorMatrix::from(canonical), authored);
    }
    for &authored in ColorRange::ALL {
        let canonical = CanonicalRange::from(authored);
        assert_eq!(ColorRange::from(canonical), authored);
    }
}
