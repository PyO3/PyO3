from setuptools_rust_extension import misc


def test_issue_219():
    # Should not deadlock
    misc.issue_219()
