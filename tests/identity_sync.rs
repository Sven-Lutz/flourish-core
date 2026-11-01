use flourish_core::*;

const USER_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const USER_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const SPACE_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";
const SPACE_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAY";
const SHARED: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAZ";
const DEVICE_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB0";
const DEVICE_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB1";
const SESSION_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB2";
const MEMBERSHIP: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB3";
const AUTH_BINDING: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB4";
const RECOVERY: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB5";

fn at(hour: u8) -> Timestamp {
    Timestamp::new(format!("2026-09-08T{hour:02}:00:00.000Z")).unwrap()
}

fn user_a() -> User {
    User::new(
        UserId::parse(USER_A).unwrap(),
        SpaceId::parse(SPACE_A).unwrap(),
        at(8),
    )
}

fn user_b() -> User {
    User::new(
        UserId::parse(USER_B).unwrap(),
        SpaceId::parse(SPACE_B).unwrap(),
        at(8),
    )
}

fn device(id: &str, user: &User, platform: ClientPlatform) -> Device {
    Device::new(
        DeviceId::parse(id).unwrap(),
        user.id().clone(),
        platform,
        "Test client".into(),
        at(8),
    )
    .unwrap()
}

fn session(user: &User, device: &Device) -> AuthSession {
    AuthSession::new(
        AuthSessionId::parse(SESSION_A).unwrap(),
        user.id().clone(),
        device,
        CredentialDigest::new([7; 32]),
        at(9),
        at(18),
    )
    .unwrap()
}

#[test]
fn flourish_identity_is_stable_and_authentication_binding_is_separate() {
    let user = user_a();
    let binding = AuthIdentityBinding::new(
        AuthIdentityBindingId::parse(AUTH_BINDING).unwrap(),
        user.id().clone(),
        AuthProvider::AppleOAuth,
        "apple-opaque-subject".into(),
        at(9),
    )
    .unwrap();

    assert_eq!(binding.user_ref(), user.id());
    assert_ne!(binding.external_subject(), user.id().id().as_str());
    assert_eq!(binding.provider(), AuthProvider::AppleOAuth);
    assert_eq!(user.personal_space_ref().id().as_str(), SPACE_A);
}

#[test]
fn all_client_modes_keep_explicit_identity_and_local_only_is_only_a_mode() {
    let user = user_a();
    let desktop = device(DEVICE_A, &user, ClientPlatform::Desktop);

    for mode in [
        IdentityRuntimeMode::LocalOnly,
        IdentityRuntimeMode::AccountOnline,
        IdentityRuntimeMode::AccountOffline,
        IdentityRuntimeMode::ReauthenticationRequired,
    ] {
        let context = ClientIdentityContext::new(
            user.id().clone(),
            user.personal_space_ref().clone(),
            desktop.id().clone(),
            mode,
        );
        assert_eq!(context.user_ref(), user.id());
        assert_eq!(context.personal_space_ref(), user.personal_space_ref());
        assert_eq!(context.device_ref(), desktop.id());
        assert_eq!(
            context.is_account_backed(),
            mode != IdentityRuntimeMode::LocalOnly
        );
    }
}

#[test]
fn session_lifecycle_fails_closed_across_user_device_expiry_and_revocation() {
    let mut user = user_a();
    let mut desktop = device(DEVICE_A, &user, ClientPlatform::Desktop);
    let other = user_b();
    assert_eq!(
        AuthSession::new(
            AuthSessionId::parse(SESSION_A).unwrap(),
            other.id().clone(),
            &desktop,
            CredentialDigest::new([1; 32]),
            at(9),
            at(18),
        )
        .unwrap_err(),
        AuthSessionViolation::DeviceUserMismatch
    );

    let mut auth = session(&user, &desktop);
    assert!(auth.is_active(&user, &desktop, &at(10)));
    assert!(!auth.is_active(&user, &desktop, &at(18)));
    auth.revoke(at(11)).unwrap();
    auth.revoke(at(12)).unwrap();
    assert!(!auth.is_active(&user, &desktop, &at(12)));

    let auth = session(&user, &desktop);
    desktop.revoke(at(12)).unwrap();
    assert!(!auth.is_active(&user, &desktop, &at(13)));

    let desktop = device(DEVICE_A, &user, ClientPlatform::Desktop);
    let auth = session(&user, &desktop);
    user.deactivate(at(12));
    assert!(!auth.is_active(&user, &desktop, &at(13)));
}

#[test]
fn recovery_challenges_are_expiring_single_use_and_store_only_a_digest() {
    let user = user_a();
    let digest = CredentialDigest::new([42; 32]);
    let mut challenge = RecoveryChallenge::new(
        RecoveryChallengeId::parse(RECOVERY).unwrap(),
        user.id().clone(),
        digest,
        at(9),
        at(10),
    )
    .unwrap();
    assert_eq!(challenge.credential_digest().as_bytes(), &[42; 32]);
    challenge.consume(at(9)).unwrap();
    assert_eq!(
        challenge.consume(at(9)).unwrap_err(),
        RecoveryChallengeViolation::AlreadyConsumed
    );

    let mut expired = RecoveryChallenge::new(
        RecoveryChallengeId::parse(RECOVERY).unwrap(),
        user.id().clone(),
        digest,
        at(9),
        at(10),
    )
    .unwrap();
    assert_eq!(
        expired.consume(at(10)).unwrap_err(),
        RecoveryChallengeViolation::Expired
    );
}

#[test]
fn personal_spaces_are_private_even_if_a_membership_is_presented() {
    let owner = user_a();
    let intruder = user_b();
    let intruder_device = device(DEVICE_B, &intruder, ClientPlatform::Mobile);
    let intruder_session = session(&intruder, &intruder_device);
    let personal = Space::new(
        owner.personal_space_ref().clone(),
        SpaceKind::Personal,
        owner.id().clone(),
        at(8),
    );

    assert_eq!(
        authorize_space_access(
            &intruder,
            &intruder_device,
            SpaceAccessRequest {
                action: AccessAction::Read,
                sensitivity: SensitivityClass::Standard,
                space: &personal,
                membership: None,
            },
            &intruder_session,
            &at(10),
        ),
        Err(AuthorizationViolation::PersonalSpaceOwnershipMismatch)
    );
    assert_eq!(
        Membership::new(
            MembershipId::parse(MEMBERSHIP).unwrap(),
            &personal,
            intruder.id().clone(),
            MembershipRole::Member,
            at(9),
        )
        .unwrap_err(),
        MembershipViolation::PersonalSpaceMembershipNotAllowed
    );
}

#[test]
fn shared_space_access_is_membership_gated_and_sensitive_domains_stay_private() {
    let owner = user_a();
    let member = user_b();
    let member_device = device(DEVICE_B, &member, ClientPlatform::Web);
    let member_session = session(&member, &member_device);
    let shared = Space::new(
        SpaceId::parse(SHARED).unwrap(),
        SpaceKind::Shared,
        owner.id().clone(),
        at(8),
    );
    assert_eq!(
        authorize_space_access(
            &member,
            &member_device,
            SpaceAccessRequest {
                action: AccessAction::Read,
                sensitivity: SensitivityClass::Standard,
                space: &shared,
                membership: None,
            },
            &member_session,
            &at(10),
        ),
        Err(AuthorizationViolation::NoActiveMembership)
    );

    let mut membership = Membership::new(
        MembershipId::parse(MEMBERSHIP).unwrap(),
        &shared,
        member.id().clone(),
        MembershipRole::Member,
        at(9),
    )
    .unwrap();

    assert_eq!(
        authorize_space_access(
            &member,
            &member_device,
            SpaceAccessRequest {
                action: AccessAction::Write,
                sensitivity: SensitivityClass::Standard,
                space: &shared,
                membership: Some(&membership),
            },
            &member_session,
            &at(10),
        ),
        Ok(())
    );
    for sensitivity in [
        SensitivityClass::ReflectivePrivate,
        SensitivityClass::WellbeingSensitive,
        SensitivityClass::BiometricHealth,
    ] {
        assert_eq!(
            authorize_space_access(
                &member,
                &member_device,
                SpaceAccessRequest {
                    action: AccessAction::Read,
                    sensitivity,
                    space: &shared,
                    membership: Some(&membership),
                },
                &member_session,
                &at(10),
            ),
            Err(AuthorizationViolation::SensitiveDomainRequiresOwner)
        );
    }

    membership.revoke(at(11)).unwrap();
    assert_eq!(
        authorize_space_access(
            &member,
            &member_device,
            SpaceAccessRequest {
                action: AccessAction::Read,
                sensitivity: SensitivityClass::Standard,
                space: &shared,
                membership: Some(&membership),
            },
            &member_session,
            &at(12),
        ),
        Err(AuthorizationViolation::NoActiveMembership)
    );
}

#[test]
fn version_vectors_detect_concurrency_instead_of_comparing_unrelated_counters() {
    let a = DeviceId::parse(DEVICE_A).unwrap();
    let b = DeviceId::parse(DEVICE_B).unwrap();
    let a_five = VersionVector::from_entries([(a.clone(), 5)]).unwrap();
    let b_one = VersionVector::initial(b.clone());

    assert_eq!(
        reconcile(&a_five, &b_one),
        ConflictOutcome::RequiresManualResolution
    );

    let downloaded = a_five.merged_with(&b_one);
    let next_b = downloaded.advanced_on(b).unwrap();
    assert_eq!(
        reconcile(&next_b, &downloaded),
        ConflictOutcome::Resolved(SyncOrdering::LocalWins)
    );
    assert_eq!(
        reconcile(&downloaded, &downloaded),
        ConflictOutcome::Equivalent
    );
}

#[test]
fn rehydrated_version_vectors_require_a_causal_entry() {
    assert_eq!(
        VersionVector::from_entries(std::iter::empty()),
        Err(flourish_core::VersionVectorViolation::Empty)
    );
}

#[test]
fn sync_metadata_separates_account_truth_local_truth_and_rebuildable_state() {
    let user = user_a();
    let desktop = device(DEVICE_A, &user, ClientPlatform::Desktop);
    let metadata = SyncMetadata::new(
        OpaqueRef::new("PURSUIT:01ARZ3NDEKTSV4RRFFQ69G5FB6").unwrap(),
        user.personal_space_ref().clone(),
        SensitivityClass::Standard,
        VersionVector::initial(desktop.id().clone()),
        SyncState::PendingUpload,
    );
    assert_eq!(metadata.space_ref(), user.personal_space_ref());
    assert_eq!(metadata.state(), SyncState::PendingUpload);

    assert!(matches!(
        DurabilityClass::AccountBacked {
            space_ref: user.personal_space_ref().clone()
        },
        DurabilityClass::AccountBacked { .. }
    ));
    assert!(matches!(
        DurabilityClass::Rebuildable,
        DurabilityClass::Rebuildable
    ));
}
