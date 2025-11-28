#[cfg(feature = "asm")]
mod assembly;
mod curve;
mod fp;
mod fq;

pub use curve::*;
pub use fp::*;
pub use fq::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ff::{Field, PrimeField};
    use group::GroupEncoding;
    use hex;
    use pasta_curves::arithmetic::CurveExt;
    use rand_chacha::ChaChaRng;
    use rand_core::{RngCore, SeedableRng};
    use subtle::ConditionallySelectable;

    const VECTORS_PER_OP: usize = 16;
    const FP_ADD_SEED: u64 = 0xaddf_00dd_eadc_0ded;
    const FP_SUB_SEED: u64 = 0x5ab5_eed0_cafe_babe;
    const FP_MUL_SEED: u64 = 0x600d_600d_bad0_f00d;
    const FP_DOUBLE_SEED: u64 = 0x00d0_00d0_00d0_00d0;
    const FP_SQUARE_SEED: u64 = 0x5a5a_5a5a_1234_5678;
    const FP_INVERT_SEED: u64 = 0x1ee1_dead_beef_cafe;
    const FP_REDUCE_SEED: u64 = 0xdec0_de01_feed_baad;
    const FQ_SEED_XOR: u64 = 0x0f0f_0f0f_0f0f_0f0f;
    const FQ_REDUCE_SEED: u64 = 0xcafe_f00d_dead_f00d;

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_misc() {
        fn log_fp(label: &str, value: &Fp) {
            println!("{label}: {}", hex::encode(value.to_repr()));
        }

        println!("step start");
        let z = Fp::ONE;
        println!("step z");
        let a = Secp256k1::a();
        println!("step a");
        let b = Secp256k1::b();
        println!("step b");
        let one = Fp::ONE;
        println!("step one");
        let three = one + one + one;
        println!("step three");
        let four = three + one;
        println!("step four");
        let z_sq = z.square();
        println!("step z_sq");
        let tmp = three * z_sq + four * a;
        println!("step tmp");

        println!("--- svdw_precomputed_constants debug ---");
        log_fp("z", &z);
        log_fp("a", &a);
        log_fp("b", &b);
        log_fp("neg one", &(-one));
        log_fp("neg one * neg one", &(-one * -one));
        log_fp("one * neg one", &(&one * &-one));
        log_fp("neg one * one", &(&-one * &one));
        log_fp("one * one", &(&one * &one));
        assert_eq!((-one * -one), one);
        assert_eq!((-one * one), -one);
        assert_eq!((one * -one), -one);
        assert_eq!((one * one), one);
    }
    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_hash_to_curve_vectors() {
        const EXPECTED: [&str; 10] = [
            "0eb2b4b4381420fade419ca60d53aaee54f93f2f1d9aa308be7c1a31932532a040",
            "296922a9c8c53e6bee8df36ad54412abcc942629e5f5fd2ddf3df90440eeed9a00",
            "35c8b4a38252656dace01fe2716e57593fab8052cd47869d30cd0a8bc24fd43600",
            "6fc90f411f3bb98bd574c5ff7529600f833993fdb698fa859365fff370fcf16340",
            "ebcd370461623827de13b2bc6e68a5b335106463f684437720e5bc859a93014100",
            "c59e5dee409716c6aad0f631155280e9c50a3b81145ab936a2979ff26579918f00",
            "119242dc506a978d992d4f1c9c0d69d825493834716e9516c4ff1aa67fe7afde40",
            "9cf860c724c7473ae7a2561cb5e5c64cbb0ac26c87b1842b86b6c053764eddd400",
            "fecfe6bf1d9dc136828b032e9cc9edbbbab97185d0355376bbaa615852f3b58540",
            "d68299006e542d36109c1a9e8749bbb129bbb3345128af002c50b70385a1d97740",
        ];

        let mut seeded_rng = ChaChaRng::seed_from_u64(0);
        let uniform_bytes = std::iter::from_fn(|| {
            let mut bytes = [0u8; 32];
            seeded_rng.fill_bytes(&mut bytes);
            Some(bytes)
        })
        .take(EXPECTED.len())
        .collect::<Vec<_>>();

        let hash = Secp256k1::hash_to_curve("from_uniform_bytes");
        for (expected, uniform) in EXPECTED.iter().zip(uniform_bytes.iter()) {
            let p = hash(uniform);
            assert_eq!(hex::encode(p.to_bytes().as_ref()), *expected);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fp_arithmetic_vectors() {
        fn encode(fp: &Fp) -> String {
            hex::encode(fp.to_repr())
        }

        const FP_ADD_EXPECTED: [&str; VECTORS_PER_OP] = [
            "1904e622282a40f2cd74ddc241078761cba52ae00e570ec8a4fbbed123d504f9",
            "85e081056fec763fb7d71ccab519e8c1d2a70ce5e86ff24320fc0b8a743a9a8b",
            "7a55c288ffbc72d95bdcee0af412124ffc109d046f2c6159f542ca501bf770c3",
            "12c9cc4017905f4dbe1fc87078f4579707bd2c4654f998796dabe3d9556f4760",
            "f29a0066ca42463d6804ab4b995274797c40cb07f16bbbbfbd47e8f22c101bd9",
            "c039ff581863ddfd069a7ebb80f6938b07d14ad6ae1c5bd4ef5d7ad8c366730a",
            "c5399063206396084769b2fecb153d183deb1cbcef70752017dcb676e3c5be8f",
            "e458c06f72a10ee5b1352b121bce7c9e9c71fd72eb51b4a82201a7ceb7455a1c",
            "3e1a3a8db113ff05d928238aec3021d58c72b1bd617b6f8e6618b3baf60e8ee8",
            "a8a9bda9f4b4bdc7fa170c5e98e33d4a75aa00033b12f1b360199d4661608b6f",
            "5f689f9957fc235b528e96c5af18fc186d55f51af15fc5265c6831e5a4e3b14e",
            "812cb8e3087a67fa0cd81ecfa89a8653083d4b63eb85b574c5b577f130548b6c",
            "d58a4a756f33c132e889e94d487d3cdacccaedfd148996461aa136c951095c5f",
            "d5bb981c491dc657ea924527b86f3941d4c9395589e8f2d71d7eb232e36f62dd",
            "c783ce1abe18df2c412e747439dae5bce3c4f68901883688bbe44bc359f5ff45",
            "fb9ea9ff18903a88ce556967b794db6ad8f4a99e48ef5fcbc8ae134f135e2b9e",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_ADD_SEED);
        for expected in FP_ADD_EXPECTED {
            let a = Fp::random(&mut rng);
            let b = Fp::random(&mut rng);
            assert_eq!(encode(&(a + b)), expected);
        }

        const FP_SUB_EXPECTED: [&str; VECTORS_PER_OP] = [
            "231f8b4d89f829cbbc275635bb2292a52c7a96ac0150f9e51f64a111dba0f0f4",
            "3e467a6c097a0c981e2bf05e182b4cebfaf8b87b45fad0190643f5e15df00277",
            "98a0824120c57f3391424cbfb81817b65febe02a600195d2f088ed90d480e9c8",
            "54772ee7f20e2ef933f66a5de3ed803089c5c3456ccfcbae9be185ce5a9c866c",
            "9d9745502a8c47cb630cb489e5ef4b89b191b6caa8c115ed506e8e9f28cfd53a",
            "ccd9e05fc81f5837055065c66cf5a3d87054cdd15cbd791c2bd0b1bb021546fc",
            "cc64cef0ae4b6752909f6472fa95d8016f1357b3d7f7e56b164957856d1962ae",
            "5f51b6ce47b6b36c412c11d5aaef28f4d39db40a761e7067254c9b2d3bb9c887",
            "97f221ce082ec45fbdd772015822706d5143ef61151f5697fde0b672e360316c",
            "ac45b70b2a04ec59c2e2a4c078cf75e6abc59e1fa78e6df9fa7630e899cc716e",
            "0ff9daac2c4bf74f7cf69fe618bd24d1d09342870a7ed77ffbe64ee8bb0e6da2",
            "ccf161c90539c5c544ec60d828328a2613af9120c47d633000d82b3e4263175c",
            "2a02724aa596866f68e71b4530f99ca9eb249fb65e8ce68249289bffa03d3891",
            "a10a2598e765cb5a019e797107140948d2b2b4059da4db98ba0074aa603e9b67",
            "46fcf74630ead36f66d3dfe0cfbf7353567dc59d3727f19f6a0c5f3e14970ac7",
            "dbf0dcb60e9d57e6f71e852b1f6b32c4fa6ebaca123c7ecbec021f3a3313a71e",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_SUB_SEED);
        for expected in FP_SUB_EXPECTED {
            let a = Fp::random(&mut rng);
            let b = Fp::random(&mut rng);
            assert_eq!(encode(&(a - b)), expected);
        }

        const FP_MUL_EXPECTED: [&str; VECTORS_PER_OP] = [
            "f6a3c61b166eb6a67be77b19c5e764597ce397f8b9faf6b25ad785fe2328fbab",
            "fccb831137f6359e112e686ab661e02af9ebce07ab867d2e70e92e7d801f15e8",
            "5ecfb6db866854f189c92ba507ed4037d6180d2575fca7f0e9330801853fff48",
            "031863ea98a983a1500195469085e7f2986d9e06973f6cbd23976a8132efb7d4",
            "552745ac250305ac2f6ffb7407284945ede34b6e672f6d9d6f6a483a1b24703f",
            "c14e80671c3cf6e5531295cc54b60ad1f58bdeccaa2e9958f0031030c79ba7df",
            "91c4413f900e25d387de2d8e121c3ad9c2b6b079366cdc51db17001edc85326c",
            "c88e35a38d3f6ae56a15360f11faa1f48f5141d9b33b2238570e57985b42ab99",
            "f3bca42fbb062cec88904569f30d0e4596e3fc8e2cb32e197ad40032b0861609",
            "69a0af36af7507231be04d090a6fcfc8b3ae8d1a6cb497b7d2a65b8f9f552937",
            "04dabb83cd10a2c8162280283af0cb2fa50abf19e38f5d04abd4becc55a8df4f",
            "9ac2ba747d27956e468a646992a14be03ac6bddb5b486e304bdc4d19047f1087",
            "1eb356c25c105d45088a58b3301e4dc34d7f3898f5acc4d6badb55c79522f073",
            "35e795e089b22556baf8ea32da0d84ce815f4a8479b82078568d99257c450315",
            "e0dcfa257184541b77f9b46a5419673ecba0e196c26e75ec8ad55551386679dc",
            "d579786597ba1f9e5aeb6c0fec5eabfd3d4f113fc5dd968c3385b835854fb1ad",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_MUL_SEED);
        for expected in FP_MUL_EXPECTED {
            let a = Fp::random(&mut rng);
            let b = Fp::random(&mut rng);
            assert_eq!(encode(&(a * b)), expected);
        }

        const FP_DOUBLE_EXPECTED: [&str; VECTORS_PER_OP] = [
            "929c009c105b88f281d937c832cb3323c72b9bfcb8366cecfb63592b423bddee",
            "606c51e0b7e4d460bc051516293434dd90aef0e673953e3b7c2f96a05d022944",
            "8f10ae45b971104acfc8050fcd7ae0be5663324c72eb7540795068288d7d9940",
            "9a3319353705cf53b4e6722e1a8e5c8159ac9fde2c7d7d32bab0f86b92c0047a",
            "2ef20463be0295f1ad43775dcc7fb26b813eee1dc726cc72b4d181a82ebe7823",
            "4d7bf3d43af307dce15b420631f67afc4100249785999ad41382b70f975d0c1d",
            "1bcbf53005c8b6c8315d5059c02876c2e09cc1654b208ea8a5514dd726ba1407",
            "db10540d646b5658a5b98593def42f82b0d26bfa63f5452462b6ddac49f15091",
            "78b433c3b1dcbc32ee0e13f4ba102891df4088274162f1ca20e9b2358e9bdbab",
            "67dbb0cd236d9f1ca58909cedc3de2a7db561929a16bbc65fa9245cbecf75efb",
            "ec55da115c7a83e9a6797bcbd3207a6eba89073c9f7b1822ddbd18ecb2305d2d",
            "8132696f8728ff6d97c654a37376fa371505436541d7ba952677bf02adebbb90",
            "77e03f8d832519803df4295b391d5263b2aa0152c25a3eb5256de4a4642a6302",
            "233e1b4d103dd721a0661bc3bd4dc9e9f530feeed4e04e7a0cb9f850db17aad6",
            "9a8fc37e0d9a5c3326a3271d50cc6c69e2be3ff6bfb6dfe7c453bffc04001262",
            "0ebd4641781fa7fc6db723f523ffb4f6dd04f6df62796a0a66f12d79f42d8bf2",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_DOUBLE_SEED);
        for expected in FP_DOUBLE_EXPECTED {
            let a = Fp::random(&mut rng);
            assert_eq!(encode(&a.double()), expected);
        }

        const FP_SQUARE_EXPECTED: [&str; VECTORS_PER_OP] = [
            "404e0b5decd1c7169da764529aa9dad5fe13957606a73e6f8993708b3e5d8a1c",
            "dd68fa75adb9dd78e4d5032c06b96a20e4259956b545e1c22623b4a0dc90e95f",
            "7f73a87cc0643789f4e963f92509d86fe3fc458cd356d526635f77bcbaea69b9",
            "88e96469c40faa5c1b64b1be232731bd268ab7e6e43d57e44c1b04af375bd68b",
            "53c8afe4bc97df4d424d332c656f2627c1690ec472ec08e915960a9b6ba6ae44",
            "919163cae00f085f8e662df8c2e5ce6640caab6ea1bd8e46831c2d44002d7b45",
            "85b1d64fde3b46c8cc6518e46eca9490c7f97567f01af707e01d3b4554e77edd",
            "3f5a184539a2d1d053f443e117bd5db07863dc25aa02ba689088fbd55ea33697",
            "42e5742e4cec6bca064be77b6fefd6b22f6cc8f1baf8c7ae5f3c40a740ba2634",
            "ac8c616473a263595e54a500412bac2d799440251854904a5fee4af56c0dcfd7",
            "5bdab2ba989a171aacdaa5dfcfd2da56e5c80c27745d79496d0798584c57ed66",
            "0e3fd86a50e88c8e82316427dc0fad02188c08db4080ede9177adf1e0d06f8a9",
            "47bfa5fc23bc21ee21eecec0d01789a005d78cc1c9df12b77c4ac77cdfad25fc",
            "c7ee06a6cd9fbed51b2c21860bef6c680507d65c6bee01971fdad24c47767c84",
            "0cd51e2af390b824ef1a5704701c7a2c257ecde6e43d9a1db1f70c309b841c74",
            "0e2b003e382415d036b7a10122c9426ac5c85a39193545ea60cfb92867776f8e",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_SQUARE_SEED);
        for expected in FP_SQUARE_EXPECTED {
            let a = Fp::random(&mut rng);
            assert_eq!(encode(&a.square()), expected);
        }

        const FP_INVERT_EXPECTED: [&str; VECTORS_PER_OP] = [
            "56b467af213a9b2e7c430617d7aa5056e3e9621dfccf8115ff29357ecdb3aee0",
            "0350f387cdca1bd303138eb7926ca997c3c019b0c61354f0288806494af3baad",
            "3403bc34c56101497010f17afc1390520909c3437fdc87120890acebfa89f37b",
            "f28dda2a728b003caedd883a9d9c6483930e366bd8c8001fd02e4e94f5cf0796",
            "eeeaad983068cec3e4066817cec886b05eaa0a673c4e3b534f69e3835f5f293b",
            "223ab65b625be2175e81033f62f3621346a814b13b512d583189a9ed3739dea8",
            "c80dac5115ea23f3c3407725cc199b19abdd1685ac54a35599569391254df3ff",
            "574e474befcb1cde4a8f8edd187d6130edce41d06bbc7cbadc50a14def78aaf3",
            "8492bd57071000dc98677f9dc0102735fcaf9fad6d45efd88c50fc9a27d9fa92",
            "c8cb8d44c690bd5642273656d92ee1ef5f8e90dadf8533ed58d319cefb3e2629",
            "dfa0a765b5d80e694f9fc51d7d8950548028a67c00289d06ed9f2d952d3c64a1",
            "aee5161c3ce6fb19819be5bb199886322929d7a8c3d3375f516ffe991ad79068",
            "77e38c050886fdb00efa91c359d98c51436042daf0bd975b539b81e8886ef413",
            "0322137d09a58bb5e206673b4e05fdf0ec23d8fba2c6993755a8aea3d57376d1",
            "52a51360beeb1fad439f481ef9ecef6812b6bfb8b0026c6c6d0f8d05fb6d7a84",
            "afdd7b7dffd0fbc5846cdcd384be1c6a35dbdee954573f7826f266530c433449",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_INVERT_SEED);
        for expected in FP_INVERT_EXPECTED {
            let val = loop {
                let candidate = Fp::random(&mut rng);
                if bool::from(candidate.is_zero()) {
                    continue;
                }
                break candidate;
            };
            let inv = val.invert().unwrap();
            assert_eq!(encode(&inv), expected);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fq_arithmetic_vectors() {
        fn encode(fq: &Fq) -> String {
            hex::encode(fq.to_repr())
        }

        const FQ_ADD_EXPECTED: [&str; VECTORS_PER_OP] = [
            "429b814ff75c9bab7b5af30984af9c70dfa2123ee55f45a179a96edb42648daf",
            "9dd251cacec8df5fea1bedb70baa7b7ee04f6c23456d442a75290361cfb0dab6",
            "7e0db59c105b807670b3162b64da145524ca0643fcdae23f1b401b9a8a8900d6",
            "986842fc32087f664d2d50e9b5d827640175c33768d09e8d78288dd38c2553f0",
            "52a7cef30aec63fbc1b388ada74825084d7adba098c1171fe62e8a12ae47d5b3",
            "0931ddc00ce34b0e8973ccd4d56601c249558570303572e3c33357c11605556c",
            "6b716ee463f73e9b8a1ce0482771c49087ba78ae1af6e6d1460b96ae48bbe867",
            "16061c608dd0bd0bce8606b088a2b403437d5301a1a470979268338312e0cf3e",
            "133fb23af3762bcbf363cafb139bd4d30a2d0ec699d1d4b1e651570c25662ae8",
            "19499dc393745b7c6d970c453a0be2bc8fb1250f46844498041b66b202df1cdc",
            "a70bccb3ebfc524a1a772393992088e073446368215a12b4c2a5a3fbcb5c02a6",
            "261e720d7c7d8bfcebcd02a25c1d608148f7ea0f60dc9456ae9fd8ae043862e2",
            "f14535ccdfdce35f04c4096e99e6a6df668a3503a9f59505c9278fb7e6a6fe8c",
            "c0b06490227a2eb4fe6caa95e2ecae0cc726c2c47963f9827e24af554db4fa8a",
            "9c91ad78b9c54c42cdb828e4b8a1bd6c6a0c9728a6542761eede3a5c44695b75",
            "6adbfc7df091408d38052c421fc0304c11b57eb8966e702322bfe60a7e2442a3",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_ADD_SEED ^ FQ_SEED_XOR);
        for expected in FQ_ADD_EXPECTED {
            let a = Fq::random(&mut rng);
            let b = Fq::random(&mut rng);
            assert_eq!(encode(&(a + b)), expected);
        }

        const FQ_SUB_EXPECTED: [&str; VECTORS_PER_OP] = [
            "8d147407fb9c418a921f3e1b4e6c43c18279491b9e9344ef031c732de2681b3a",
            "dcdb905586acb14fab129ff0e17fc932992ece71a4daa2342f6fe6dea5171844",
            "f0358f099757a63a51ed3c7699d3ed48f20e2bd2e40d1ce848a32c664ced91a8",
            "312408f783220d81ec347b4061d0542cee8ba63fbd982060e89b364487939c0a",
            "573b20bd543670777fcdec00fe30d78d14ad70647ee510d506aed12056bba404",
            "667f31828d17a9841b5482ebdf4ff5e9818e13f67b62b26b65186f3a164af569",
            "eae8e3473e5c5dbced19dfb428ca349589e6e93dbd5a08f4b47118059996bcd0",
            "5e715819b2b20013bba042a318826cd1d54044d423fe54571210b487e94826e8",
            "526cbdc17d824dc17f4d6934078526a315c433eccc876b164bb6db74829bcaf4",
            "43e7115f444555dac02d983f896d16994d4482bb2365e8e1c0f0d9ea41e22628",
            "d753737ea4cde14917c1d5c1043ab320bd3ebec0bf9251cc8f29c4bab94c9f4d",
            "33ce5aa3b84e9a1f923457c80531de98a405945b0f4c55a0ee2b4093a6de2df9",
            "1d28c7a853ee9743cc84a802f829a31fbb364770126a6f825ff5f13d9656d640",
            "68a020ff45e80b55286c294a6b64e0c92d7039780f309d69844baae4528d903e",
            "838b9a3eae3c07c7689d58efd61685f6a96ab4c8165f4f92e2bcc6643626d443",
            "57939184c2581682b411118a42a31f9dea64875ef04b70ecf1d6d716a1a6edb4",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_SUB_SEED ^ FQ_SEED_XOR);
        for expected in FQ_SUB_EXPECTED {
            let a = Fq::random(&mut rng);
            let b = Fq::random(&mut rng);
            assert_eq!(encode(&(a - b)), expected);
        }

        const FQ_MUL_EXPECTED: [&str; VECTORS_PER_OP] = [
            "2069b7cf891a267c4634a4ae10c7897a3c503976f5af17f1e92eb972c0a57311",
            "4ee823726ab398531438a0228933af16ee87cc0c30cbd1140ec02fd6181ccad8",
            "be135adeeed1d333c799df85601aa95953288f7d4fb5458eb9b08cb468ad963f",
            "515839fbc4efb5da362b8c9ccbe5cb7ba3d8eb1e8929de12f50d2d809e847a47",
            "82bf1f6c44c2ace76eadd3af4d3bbf1dbc62233b8b64b9115e7fa550a3378e44",
            "bda61384b2c48c713e79e1cfc6d6d5d322dd70368af8c485eb41599e31c3cede",
            "c33daf45fdc032b941d433766e8e087f6bc1bbfb2afd970104b4dd21a7651e51",
            "e32e2dd87767b805d7767929aba95adf9da6e69040458fbd930b0206caaeeb0f",
            "1479f870a9b4cab81319d5c9c9550c5575f07fba5bfc96540eee08f593800771",
            "aad285ed9ff214c9defd19421da1f360081b10a1c5d283a2cab3f65f450e13e0",
            "7bcb6c3a292eccd3bf50c7232fe3f0d3ad6a65db1901c930aa98df08c32ee26c",
            "9aafa8c15790c4296ab0cd58722b5bf4d37ec22400ef5b106d3f92bc8908cf87",
            "36f8bf915bf25c4e6136064a3dec76b6f9bf8bf78da1fdb8010db79ac18501d2",
            "0f25261dfe6c4e58a81e433fbf196e8874f84aacd32872b63c4ac889a78c8741",
            "25408af146888cb17dcf31555739a4e0613fd17f64ec183bc071cf751a94ae38",
            "e86c7af9eb603b2e9275a2453d600f0f37a00865517cc25d841a9910d777f677",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_MUL_SEED ^ FQ_SEED_XOR);
        for expected in FQ_MUL_EXPECTED {
            let a = Fq::random(&mut rng);
            let b = Fq::random(&mut rng);
            assert_eq!(encode(&(a * b)), expected);
        }

        const FQ_DOUBLE_EXPECTED: [&str; VECTORS_PER_OP] = [
            "4cead47e7601dbbb224643cf4787a69c87c90d657426c632f4b4802603e785e3",
            "92da8d3babbc08b6933e83e6142a0505ac3cf10d480a53a57f5ce5ac0f8f811a",
            "a5280ff6ce36e821445f66e2f93a2fa7712c920a35ba8088d3e809ec1d91510a",
            "c3ce972b3616d5f179937d6a89e6af4bfdb6d48dfdd55b6a67623a5de56c0a98",
            "6ceec691989810678872b1e66e8cac61bc39ea954fb174b5bb8364264a96cb49",
            "d2da168f9946bdf943fc3dc818b3cce45b01db6aa384510ee7babeba6aca6056",
            "2ad8328d752d1410375d3d50d5f523abdd64411c4946735f5e634317dfc0445b",
            "a41347ee6baa7b5006fb5326c3637bcc5a8e50c685c9e131d6f5fa427ceb828f",
            "fd88066d24cb58c2efe18f01eae57f5fbff25cd0ab395881398841c5de2b2544",
            "01cde85cd92afef71aa9de29d268e0156ab5639b5ed626eb38d8a2dd220491a5",
            "5888d110ac57419db653b4b2d6d94eeed37387ba76ac75a5c397abd2dd635b70",
            "9f03e154e8d92632af9dd347a1b16f166ef96192f7b3e37b5c981a9c89ef9623",
            "44743f05ea90cb2d9880e73cebd4887cddfeb02d2ec4713e499bd17c8bfff849",
            "723b2f1a6876f6e666629a5ad08f6593c53a88a0896184d995d54d35a34dc867",
            "01e489c49de36b4bb37cb52cc21221515caa5f11ebd9f2d4c4a880032192d5da",
            "76e023deaaee2fd32812ec80b6400c5a10d5c6dc7564a5359acbe6b917044a18",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_DOUBLE_SEED ^ FQ_SEED_XOR);
        for expected in FQ_DOUBLE_EXPECTED {
            let a = Fq::random(&mut rng);
            assert_eq!(encode(&a.double()), expected);
        }

        const FQ_SQUARE_EXPECTED: [&str; VECTORS_PER_OP] = [
            "147cd8e325c39f42c2187a26a547d2a8b94aacb573caaa95458c7606acd383ae",
            "ae65dd5cd2b3ca932d9b44c64177490c60dffb15879aaccf24e89e9a89af171c",
            "117ce7d35853a70ad8768d78e054d17fc9045fc9a728e4156ce8bf979da4215f",
            "50d939ed72a5c683f161d31795d0167f1641e7b272d206e2b99333649eee1b8f",
            "221a9ff340db27bd9c5f1fef61268c0ea8ddbbfd1ecb6f6646e8d95eaf7dd363",
            "261ee4d3c9d52d4c5da4e3315a962eb154c1222575831a9aacef9ba005b5e92d",
            "8d3d4aaf2ccd14c36eb1617e08591f13483a9d6a4f4b295648292922de6b65e9",
            "b573c459acf1bdd28ada698f163ea70c138101c4777a01c9355c9d16acb2bd25",
            "09b4bba1058c201375f0007c805070052b424afe6fbe90db853940ca42a75d30",
            "c57f8b0bffdd19332c7459ce344319e576d7a44458d8dc7541b4ce8dacf7fa73",
            "6cde83d9ee732f0fc9bc8d657f842224c712681f70e43d8f3c28953d8aeb296a",
            "11063ee9fca21c799a1692f6051925b8ecad8cc6e52133df68aa25d8742d3401",
            "2abfa1a349bafeb6c37badac16031596bf69f25eabed7c5cf0fb4027f0cd6825",
            "f4811c406c5959847afcde648198986c3011c81643d8d626594d075ea698a791",
            "b9c22cc6b91a1dada116937e8d3747c7a1e47e5c9eb0b547b651b0d0926984b8",
            "e5d10b32404b9963da712eb86dba351d8c161da5ed52b8da10dfb904580ed474",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_SQUARE_SEED ^ FQ_SEED_XOR);
        for expected in FQ_SQUARE_EXPECTED {
            let a = Fq::random(&mut rng);
            assert_eq!(encode(&a.square()), expected);
        }

        const FQ_INVERT_EXPECTED: [&str; VECTORS_PER_OP] = [
            "43f97f007a92c0a7685f198bad5e715fccccf33b2eb7154088a44001e6b6b339",
            "ada03ecc1465ba85a92525d0a2c9e1e92419ac83af50e991553e90a6ea5f44e4",
            "605b262de66047dea3b060e7f682fecddc7a96d2e1a83dbe243bd74011b715ca",
            "fff641e65b0d9960b641940b7f8dac5d2b75b63349b6864cc692ff508840c990",
            "e9935f9dc9b6f5b02e15f5137938637801e2ff20af010f824779aee2841608ae",
            "4b0dec2fd81a9b0cf4d275aeb8a1d6ef3f887b04b31ca5d117f245f634470190",
            "7617e467d1058754efdea3b1e04a240b03ad4723e92c3f6361c00a3fa0bc753e",
            "ba1e3a555f14d7128417b1a10e6df9715269aa6f921f0bfcdf3d2a1364ab22bc",
            "f1e30c059bfc8fc306c6f3b10fb74de99b6f51c2cdafeeb6e02dbbe3b8901236",
            "5287396a67b3412ea658ce8a0407d515c4e1775ebc11a21645a62bd626ee22ee",
            "9a3c21ebce582e6361a3bba860051a15ae1d60faeca97dfbc79e797d906a1908",
            "ee97d50a68fadc7a3e14d871b22d7e550e2c319ca2918aef0032bab76a4cefc1",
            "d2d2e1a69776b60223a41771fd9ded7907aa212eac4f03c5d5ddddec7775c949",
            "1ca01c28802a32be9b69288ddfad3a5f091fb85e48f377520253e70fb59444a7",
            "f5dae06532ab7549568693b7d4db1883ea3488121f42fcf7f7a26ef5f496c9ac",
            "47257fe44e7ed092f9522a5c5d4ec585a0117449bfedbf1978be1ddc2020e302",
        ];
        let mut rng = ChaChaRng::seed_from_u64(FP_INVERT_SEED ^ FQ_SEED_XOR);
        for expected in FQ_INVERT_EXPECTED {
            let val = loop {
                let candidate = Fq::random(&mut rng);
                if bool::from(candidate.is_zero()) {
                    continue;
                }
                break candidate;
            };
            let inv = val.invert().unwrap();
            assert_eq!(encode(&inv), expected);
        }
    }

    #[cfg(feature = "asm")]
    fn montgomery_reduce_dense(r: [u64; 8], modulus: &[u64; 4], inv: u64) -> [u64; 4] {
        use crate::arithmetic::{adc, mac, sbb};
        let (mut r0, mut r1, mut r2, mut r3, mut r4, mut r5, mut r6, mut r7) =
            (r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]);
        let mut carry2 = 0u64;

        let k = r0.wrapping_mul(inv);
        let (_, mut carry) = mac(r0, k, modulus[0], 0);
        (r1, carry) = mac(r1, k, modulus[1], carry);
        (r2, carry) = mac(r2, k, modulus[2], carry);
        (r3, carry) = mac(r3, k, modulus[3], carry);
        (r4, carry2) = adc(r4, 0, carry);

        let k = r1.wrapping_mul(inv);
        let (_, mut carry) = mac(r1, k, modulus[0], 0);
        (r2, carry) = mac(r2, k, modulus[1], carry);
        (r3, carry) = mac(r3, k, modulus[2], carry);
        (r4, carry) = mac(r4, k, modulus[3], carry);
        (r5, carry2) = adc(r5, carry2, carry);

        let k = r2.wrapping_mul(inv);
        let (_, mut carry) = mac(r2, k, modulus[0], 0);
        (r3, carry) = mac(r3, k, modulus[1], carry);
        (r4, carry) = mac(r4, k, modulus[2], carry);
        (r5, carry) = mac(r5, k, modulus[3], carry);
        (r6, carry2) = adc(r6, carry2, carry);

        let k = r3.wrapping_mul(inv);
        let (_, mut carry) = mac(r3, k, modulus[0], 0);
        (r4, carry) = mac(r4, k, modulus[1], carry);
        (r5, carry) = mac(r5, k, modulus[2], carry);
        (r6, carry) = mac(r6, k, modulus[3], carry);
        (r7, carry2) = adc(r7, carry2, carry);

        let (d0, borrow) = sbb(r4, modulus[0], 0);
        let (d1, borrow) = sbb(r5, modulus[1], borrow);
        let (d2, borrow) = sbb(r6, modulus[2], borrow);
        let (d3, borrow) = sbb(r7, modulus[3], borrow);
        let (_, borrow_flag) = sbb(carry2, 0, borrow);

        let (d0, carry) = adc(d0, modulus[0] & borrow_flag, 0);
        let (d1, carry) = adc(d1, modulus[1] & borrow_flag, carry);
        let (d2, carry) = adc(d2, modulus[2] & borrow_flag, carry);
        let (d3, _) = adc(d3, modulus[3] & borrow_flag, carry);

        [d0, d1, d2, d3]
    }

    #[cfg(feature = "asm")]
    fn reference_montgomery_reduce_short(r: [u64; 4], modulus: &[u64; 4], inv: u64) -> [u64; 4] {
        use crate::arithmetic::{adc, mac, macx, sbb};
        let (mut r0, mut r1, mut r2, mut r3) = (r[0], r[1], r[2], r[3]);

        let k0 = r0.wrapping_mul(inv);
        let (_, mut carry) = macx(r0, k0, modulus[0]);
        (r1, carry) = mac(r1, k0, modulus[1], carry);
        (r2, carry) = mac(r2, k0, modulus[2], carry);
        (r3, carry) = mac(r3, k0, modulus[3], carry);
        r0 = carry;

        let k1 = r1.wrapping_mul(inv);
        let (_, mut carry) = macx(r1, k1, modulus[0]);
        (r2, carry) = mac(r2, k1, modulus[1], carry);
        (r3, carry) = mac(r3, k1, modulus[2], carry);
        (r0, carry) = mac(r0, k1, modulus[3], carry);
        r1 = carry;

        let k2 = r2.wrapping_mul(inv);
        let (_, mut carry) = macx(r2, k2, modulus[0]);
        (r3, carry) = mac(r3, k2, modulus[1], carry);
        (r0, carry) = mac(r0, k2, modulus[2], carry);
        (r1, carry) = mac(r1, k2, modulus[3], carry);
        r2 = carry;

        let k3 = r3.wrapping_mul(inv);
        let (_, mut carry) = macx(r3, k3, modulus[0]);
        (r0, carry) = mac(r0, k3, modulus[1], carry);
        (r1, carry) = mac(r1, k3, modulus[2], carry);
        (r2, carry) = mac(r2, k3, modulus[3], carry);
        r3 = carry;

        let (d0, borrow) = sbb(r0, modulus[0], 0);
        let (d1, borrow) = sbb(r1, modulus[1], borrow);
        let (d2, borrow) = sbb(r2, modulus[2], borrow);
        let (d3, borrow) = sbb(r3, modulus[3], borrow);

        let (d0, carry) = adc(d0, modulus[0] & borrow, 0);
        let (d1, carry) = adc(d1, modulus[1] & borrow, carry);
        let (d2, carry) = adc(d2, modulus[2] & borrow, carry);
        let (d3, _) = adc(d3, modulus[3] & borrow, carry);

        [d0, d1, d2, d3]
    }

    #[cfg(feature = "asm")]
    fn reference_mul(a: &[u64; 4], b: &[u64; 4], modulus: &[u64; 4], inv: u64) -> [u64; 4] {
        let product = crate::arithmetic::mul_512(*a, *b);
        montgomery_reduce_dense(product, modulus, inv)
    }

    #[cfg(feature = "asm")]
    fn reference_add_mod(a: &[u64; 4], b: &[u64; 4], modulus: &[u64; 4]) -> [u64; 4] {
        use crate::arithmetic::{adc, sbb};
        let (d0, carry0) = adc(a[0], b[0], 0);
        let (d1, carry1) = adc(a[1], b[1], carry0);
        let (d2, carry2) = adc(a[2], b[2], carry1);
        let (d3, carry3) = adc(a[3], b[3], carry2);

        let (d0_sub, borrow0) = sbb(d0, modulus[0], 0);
        let (d1_sub, borrow1) = sbb(d1, modulus[1], borrow0);
        let (d2_sub, borrow2) = sbb(d2, modulus[2], borrow1);
        let (d3_sub, borrow3) = sbb(d3, modulus[3], borrow2);
        let (_, borrow_mask) = sbb(carry3, 0, borrow3);

        let (d0_full, carry) = adc(d0_sub, modulus[0] & borrow_mask, 0);
        let (d1_full, carry) = adc(d1_sub, modulus[1] & borrow_mask, carry);
        let (d2_full, carry) = adc(d2_sub, modulus[2] & borrow_mask, carry);
        let (d3_full, _) = adc(d3_sub, modulus[3] & borrow_mask, carry);

        [d0_full, d1_full, d2_full, d3_full]
    }

    #[cfg(feature = "asm")]
    fn reference_sub_mod(a: &[u64; 4], b: &[u64; 4], modulus: &[u64; 4]) -> [u64; 4] {
        use crate::arithmetic::{adc, sbb};
        let (d0, borrow0) = sbb(a[0], b[0], 0);
        let (d1, borrow1) = sbb(a[1], b[1], borrow0);
        let (d2, borrow2) = sbb(a[2], b[2], borrow1);
        let (d3, borrow3) = sbb(a[3], b[3], borrow2);

        let mask = borrow3;
        let (d0_full, carry) = adc(d0, modulus[0] & mask, 0);
        let (d1_full, carry) = adc(d1, modulus[1] & mask, carry);
        let (d2_full, carry) = adc(d2, modulus[2] & mask, carry);
        let (d3_full, _) = adc(d3, modulus[3] & mask, carry);

        [d0_full, d1_full, d2_full, d3_full]
    }

    #[cfg(feature = "asm")]
    fn reference_double(a: &[u64; 4], modulus: &[u64; 4]) -> [u64; 4] {
        reference_add_mod(a, a, modulus)
    }

    #[cfg(feature = "asm")]
    fn reference_neg(a: &[u64; 4], modulus: &[u64; 4]) -> [u64; 4] {
        use crate::arithmetic::sbb;
        if a.iter().all(|&limb| limb == 0) {
            return [0; 4];
        }
        let (d0, borrow0) = sbb(modulus[0], a[0], 0);
        let (d1, borrow1) = sbb(modulus[1], a[1], borrow0);
        let (d2, borrow2) = sbb(modulus[2], a[2], borrow1);
        let (d3, _) = sbb(modulus[3], a[3], borrow2);
        [d0, d1, d2, d3]
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fp_basic_ops_match_reference() {
        use crate::arithmetic::mul_512;
        const MODULUS: [u64; 4] = [
            0xfffffffefffffc2f,
            0xffffffffffffffff,
            0xffffffffffffffff,
            0xffffffffffffffff,
        ];
        const INV: u64 = 0xd838091dd2253531;
        let mut rng = ChaChaRng::seed_from_u64(0xabadcafe0011);
        for _ in 0..128 {
            let a = Fp::random(&mut rng);
            let b = Fp::random(&mut rng);
            assert_eq!((a + b).0, reference_add_mod(&a.0, &b.0, &MODULUS));
            assert_eq!((a - b).0, reference_sub_mod(&a.0, &b.0, &MODULUS));
            assert_eq!(a.double().0, reference_double(&a.0, &MODULUS));
            assert_eq!((-a).0, reference_neg(&a.0, &MODULUS));
            let product = mul_512(a.0, b.0);
            let expected_square = montgomery_reduce_dense(mul_512(a.0, a.0), &MODULUS, INV);
            assert_eq!(a.square().0, expected_square);
            let expected_mul = montgomery_reduce_dense(product, &MODULUS, INV);
            assert_eq!(a.mul(&b).0, expected_mul);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fq_basic_ops_match_reference() {
        use crate::arithmetic::mul_512;
        const MODULUS: [u64; 4] = [
            0xbfd25e8cd0364141,
            0xbaaedce6af48a03b,
            0xfffffffffffffffe,
            0xffffffffffffffff,
        ];
        const INV: u64 = 0x4b0dff665588b13f;
        let mut rng = ChaChaRng::seed_from_u64(0xfacefeed7788);
        for _ in 0..128 {
            let a = Fq::random(&mut rng);
            let b = Fq::random(&mut rng);
            assert_eq!((a + b).0, reference_add_mod(&a.0, &b.0, &MODULUS));
            assert_eq!((a - b).0, reference_sub_mod(&a.0, &b.0, &MODULUS));
            assert_eq!(a.double().0, reference_double(&a.0, &MODULUS));
            assert_eq!((-a).0, reference_neg(&a.0, &MODULUS));
            let expected_square = montgomery_reduce_dense(mul_512(a.0, a.0), &MODULUS, INV);
            assert_eq!(a.square().0, expected_square);
            let expected_mul = montgomery_reduce_dense(mul_512(a.0, b.0), &MODULUS, INV);
            assert_eq!(a.mul(&b).0, expected_mul);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fp_mul_matches_reference() {
        const MODULUS: [u64; 4] = [
            0xfffffffefffffc2f,
            0xffffffffffffffff,
            0xffffffffffffffff,
            0xffffffffffffffff,
        ];
        const INV: u64 = 0xd838091dd2253531;
        let mut rng = ChaChaRng::seed_from_u64(0xabad1dea12345678);
        for _ in 0..128 {
            let a = Fp::random(&mut rng);
            let b = Fp::random(&mut rng);
            let expected = Fp(reference_mul(&a.0, &b.0, &MODULUS, INV));
            assert_eq!(a.mul(&b), expected);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fq_mul_matches_reference() {
        const MODULUS: [u64; 4] = [
            0xbfd25e8cd0364141,
            0xbaaedce6af48a03b,
            0xfffffffffffffffe,
            0xffffffffffffffff,
        ];
        const INV: u64 = 0x4b0dff665588b13f;
        let mut rng = ChaChaRng::seed_from_u64(0xfeedcafe42);
        for _ in 0..128 {
            let a = Fq::random(&mut rng);
            let b = Fq::random(&mut rng);
            let expected = Fq(reference_mul(&a.0, &b.0, &MODULUS, INV));
            assert_eq!(a.mul(&b), expected);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fp_montgomery_reduce_256_matches_reference() {
        const MODULUS: [u64; 4] = [
            0xfffffffefffffc2f,
            0xffffffffffffffff,
            0xffffffffffffffff,
            0xffffffffffffffff,
        ];
        const INV: u64 = 0xd838091dd2253531;
        let mut rng = ChaChaRng::seed_from_u64(FP_REDUCE_SEED);

        let deterministic = [[0u64; 4], MODULUS, [0x1, 0x0, 0x0, 0x0], [u64::MAX; 4]];

        for limbs in deterministic {
            let expected = reference_montgomery_reduce_short(limbs, &MODULUS, INV);
            let actual = Fp(limbs).montgomery_reduce_256().0;
            assert_eq!(actual, expected);
        }

        for _ in 0..128 {
            let limbs = [
                rng.next_u64(),
                rng.next_u64(),
                rng.next_u64(),
                rng.next_u64(),
            ];
            let expected = reference_montgomery_reduce_short(limbs, &MODULUS, INV);
            let actual = Fp(limbs).montgomery_reduce_256().0;
            assert_eq!(actual, expected);
        }
    }

    #[cfg(feature = "asm")]
    #[test]
    fn test_secp256k1_fq_montgomery_reduce_256_matches_reference() {
        const MODULUS: [u64; 4] = [
            0xbfd25e8cd0364141,
            0xbaaedce6af48a03b,
            0xfffffffffffffffe,
            0xffffffffffffffff,
        ];
        const INV: u64 = 0x4b0dff665588b13f;
        let mut rng = ChaChaRng::seed_from_u64(FQ_REDUCE_SEED);

        let deterministic = [[0u64; 4], MODULUS, [0x1, 0x0, 0x0, 0x0], [u64::MAX; 4]];

        for limbs in deterministic {
            let expected = reference_montgomery_reduce_short(limbs, &MODULUS, INV);
            let actual = Fq(limbs).montgomery_reduce_256().0;
            assert_eq!(actual, expected);
        }

        for _ in 0..128 {
            let limbs = [
                rng.next_u64(),
                rng.next_u64(),
                rng.next_u64(),
                rng.next_u64(),
            ];
            let expected = reference_montgomery_reduce_short(limbs, &MODULUS, INV);
            let actual = Fq(limbs).montgomery_reduce_256().0;
            assert_eq!(actual, expected);
        }
    }
}
