# 10. 캐릭터 팩 규격

## 폴더 구조

```text
characters/<character-id>/
├─ manifest.json
├─ idle/
├─ walk/
├─ dragged/
├─ thrown/
├─ kick/
├─ speak/
├─ chase/
├─ dance/
├─ carry-card/
├─ click-react/
├─ special/
└─ accessories/
```

## 상태 요구사항

필수:

- idle
- walk
- dragged
- thrown
- kick
- speak

권장:

- chase
- dance
- carry-card
- click-react
- special

필수 상태가 없으면 내장 placeholder의 해당 상태를 사용한다. 권장 상태가 없으면 idle 또는 walk로 대체한다.

## 이미지 규격

- PNG 또는 WebP
- 투명 배경
- 논리 캔버스 256×256
- 모든 프레임의 캔버스 크기와 기준점 동일
- 방향 전환은 좌우 반전 가능
- 액세서리는 본체와 같은 캔버스의 별도 투명 레이어
- 그림자, 말풍선, 카드와 본체 자산 분리

## manifest 예시

```json
{
  "schemaVersion": 1,
  "id": "michin-potato",
  "displayName": "미친감자",
  "canvas": { "width": 256, "height": 256 },
  "anchors": {
    "feet": { "x": 128, "y": 236 },
    "speech": { "x": 128, "y": 20 },
    "accessory": { "x": 128, "y": 128 },
    "kickImpact": { "x": 228, "y": 190 }
  },
  "hitbox": { "x": 42, "y": 30, "width": 172, "height": 210 },
  "animations": {
    "idle": {
      "frames": ["idle/0.webp", "idle/1.webp"],
      "frameMs": 500,
      "loop": true
    },
    "walk": {
      "frames": ["walk/0.webp", "walk/1.webp"],
      "frameMs": 140,
      "loop": true
    },
    "kick": {
      "frames": ["kick/0.webp", "kick/1.webp", "kick/2.webp"],
      "frameMs": 100,
      "loop": false,
      "impactFrame": 1
    }
  },
  "accessories": []
}
```

## 검증 규칙

- schemaVersion 지원 여부
- ID 중복 없음
- canvas 양수, 권장 256×256
- 모든 경로가 캐릭터 폴더 안에 존재
- frameMs 16~5,000ms
- impactFrame이 frames 범위 안에 있음
- anchor와 hitbox가 canvas 안에 있음
- `..` 또는 절대 경로 금지
- 비정상 파일이면 앱 시작을 막지 않고 placeholder 사용

## 자산 교체 절차

1. 새 폴더와 manifest 생성
2. 검증 스크립트 실행
3. 모든 필수 상태 preview
4. walk 시 feet anchor 흔들림 확인
5. kick impact frame 확인
6. 50%, 100%, 200% scale 확인
7. placeholder를 fallback으로 유지

## 권리 확인

- WinDbg/Hyper-V 공식 로고는 재배포 권리가 확인될 때만 사용
- 불명확하면 자체 제작한 `DBG`, `VM` 테마 이미지 사용
- 외부 캐릭터, 폰트와 효과음의 출처와 라이선스 기록
