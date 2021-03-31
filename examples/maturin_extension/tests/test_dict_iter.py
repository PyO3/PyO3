import pytest
import maturin_extension.dict_iter as rdi


@pytest.mark.parametrize("size", [64, 128, 256])
def test_size(size):
    d = {}
    for i in range(size):
        d[i] = str(i)
    assert rdi.DictSize(len(d)).iter_dict(d) == size
