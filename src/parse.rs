/// This helper function parses a NEVRA string into its components.
#[allow(clippy::many_single_char_names)]
pub fn parse_nevra(nevra: &str) -> Result<(&str, &str, &str, &str, &str), String> {
    let mut nevr_a: Vec<&str> = nevra.rsplitn(2, '.').collect();

    if nevr_a.len() != 2 {
        return Err(format!("Unexpected error when parsing NEVRAs: {}", nevra));
    };

    // rsplitn returns things in reverse order
    let a = nevr_a.remove(0);
    let nevr = nevr_a.remove(0);

    let mut n_ev_r: Vec<&str> = nevr.rsplitn(3, '-').collect();

    if n_ev_r.len() != 3 {
        return Err(format!("Unexpected error when parsing NEVRAs: {}", nevr));
    };

    // rsplitn returns things in reverse order
    let r = n_ev_r.remove(0);
    let ev = n_ev_r.remove(0);
    let n = n_ev_r.remove(0);

    let (e, v) = if ev.contains(':') {
        let mut e_v: Vec<&str> = ev.split(':').collect();
        let e = e_v.remove(0);
        let v = e_v.remove(0);
        (e, v)
    } else {
        ("0", ev)
    };

    Ok((n, e, v, r, a))
}

/// This helper function parses a NEVRA.rpm string into its components.
#[allow(clippy::many_single_char_names)]
pub fn parse_filename(nevrax: &str) -> Result<(&str, &str, &str, &str, &str), String> {
    let mut nevra_x: Vec<&str> = nevrax.rsplitn(2, '.').collect();

    if nevra_x.len() != 2 {
        return Err(format!("Unexpected error when parsing dnf output: {}", nevrax));
    };

    // rsplitn returns things in reverse order
    let _x = nevra_x.remove(0);
    let nevra = nevra_x.remove(0);

    let (n, e, v, r, a) = parse_nevra(nevra)?;
    Ok((n, e, v, r, a))
}

/// This helper function parses an NVR string into its components.
#[allow(clippy::many_single_char_names)]
pub fn parse_nvr(nvr: &str) -> Result<(&str, &str, &str), String> {
    let mut n_v_r: Vec<&str> = nvr.rsplitn(3, '-').collect();

    if n_v_r.len() != 3 {
        return Err(format!("Unexpected error when parsing NEVRAs: {}", nvr));
    };

    // rsplitn returns things in reverse order
    let r = n_v_r.remove(0);
    let v = n_v_r.remove(0);
    let n = n_v_r.remove(0);

    Ok((n, v, r))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn nevra() {
        let string = "maven-1:3.6.1-5.fc32.noarch";
        let value = ("maven", "1", "3.6.1", "5.fc32", "noarch");

        assert_eq!(parse_nevra(string).unwrap(), value);

        let string = "dnf-4.2.18-2.fc32.noarch";
        let value = ("dnf", "0", "4.2.18", "2.fc32", "noarch");

        assert_eq!(parse_nevra(string).unwrap(), value);
    }

    #[test]
    fn filename() {
        let string = "maven-1:3.6.1-5.fc32.noarch.rpm";
        let value = ("maven", "1", "3.6.1", "5.fc32", "noarch");

        assert_eq!(parse_filename(string).unwrap(), value);

        let string = "dnf-4.2.18-2.fc32.src.rpm";
        let value = ("dnf", "0", "4.2.18", "2.fc32", "src");

        assert_eq!(parse_filename(string).unwrap(), value);
    }

    #[test]
    fn nvr() {
        let string = "maven-3.6.1-5.fc32";
        let value = ("maven", "3.6.1", "5.fc32");

        assert_eq!(parse_nvr(string).unwrap(), value);

        let string = "dnf-4.2.18-2.fc32";
        let value = ("dnf", "4.2.18", "2.fc32");

        assert_eq!(parse_nvr(string).unwrap(), value);
    }
}
