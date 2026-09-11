// Copyright (C) 2025 Kinet Labs, Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the Apache-2.0 license as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// Apache-2.0 license for more details.
//
// You should have received a copy of the Apache-2.0 license
// along with this program.  If not, see <http://www.apache.org/licenses//>.

#[cfg(test)]
mod test {
    use kinet_mock_swarm::swarm_relation::KinetMessageNoSerSwarm;
    use kinet_twins_utils::{run_twins_test, twin_reader::read_twins_test};
    use test_case::test_case;

    const TWIN_DEFAULT_SEED: u64 = 1;

    #[test_case("./tests/happy_path.json"; "happy_path")]
    #[test_case("./tests/one_twin.json"; "one_twin")]
    #[test_case("./tests/one_twin_partition.json"; "one_twin_partition")]
    #[test_case("./tests/make_progress.json"; "make_progress")]

    fn twins_testing(path: &str) {
        let test_case = read_twins_test::<KinetMessageNoSerSwarm>(path);

        println!(
            "running twins_testing, description: {:?}",
            test_case.description,
        );

        run_twins_test::<_, _, _, KinetMessageNoSerSwarm>(TWIN_DEFAULT_SEED, test_case)
    }

    #[should_panic]
    #[test_case("./tests/too_much_twin.json"; "too_much_twin")]
    #[test_case("./tests/too_much_twin_with_big_delay.json"; "too_much_twin_with_big_delay")]
    #[test_case("./tests/mal_formed.json"; "mal_formed json")]

    fn twins_should_fail_testing(path: &str) {
        let test_case = read_twins_test::<KinetMessageNoSerSwarm>(path);
        println!(
            "running expected fail twins_testing, description: {:?}",
            test_case.description
        );

        run_twins_test::<_, _, _, KinetMessageNoSerSwarm>(TWIN_DEFAULT_SEED, test_case)
    }
}
