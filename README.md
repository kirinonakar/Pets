# Rust Desktop Pets (GP-Chan & GEMMI-Chan)

<img src="rust-pets/app.png" width="50%" />

Rust로 작성된 고성능 GPU 가속 데스크톱 펫 애플리케이션입니다. GP-Chan과 GEMMI-Chan 캐릭터를 통합하여 지원하며, 실시간 시스템 리소스 모니터링 기능을 제공합니다.

본 프로젝트는 아래의 원본 Python 프로젝트를 **Rust**로 포팅하고 기능을 통합 및 개선한 버전입니다:
- [GP-Chan](https://github.com/gpgpchan-svg/GP-Chan.git)
- [GEMMI-Chan](https://github.com/gpgpchan-svg/GEMMI-Chan.git)

## ✨ 주요 기능

- **캐릭터 전환**: 우클릭 메뉴를 통해 **GP-Chan**과 **GEMMI-Chan** 사이를 실시간으로 전환할 수 있습니다.
- **시스템 리소스 시각화**: CPU, RAM, GPU, VRAM 사용량을 색상별 **그래프 바**로 직관적으로 표시합니다.
- **마우스 상호작용**: 캐릭터가 마우스 커서의 위치를 따라 좌우를 바라봅니다 (설정에서 켜기/끄기 가능).
- **드래그 앤 드롭 이동**: 캐릭터를 클릭하여 드래그하면 윈도우 전체를 자유롭게 이동할 수 있습니다.
- **부드러운 애니메이션**: `egui`를 활용한 고성능 스프라이트 애니메이션 시스템.
- **세련된 UI**: 투명 배경 및 '항상 위' 설정을 지원하는 깔끔한 오버레이 창.

## 🚀 시작하기
### 📥 Download
You can download the latest version from the [Releases Page](https://github.com/kirinonakar/Pets/releases).


### 사전 요구 사항

- **Rust**: 최신 안정 버전의 Rust가 설치되어 있어야 합니다.
- **NVIDIA GPU**: GPU 모니터링 기능을 위해 NVIDIA 드라이버가 필요합니다.
- **Windows OS**: 현재 Windows 환경에 최적화되어 있습니다 (폰트 경로 및 윈도우 관리).

### 설치 및 실행

1. 프로젝트 폴더 생성 및 이동:
   ```bash
   mkdir Pets
   cd Pets
   ```
2. 원본 Python 프로젝트 및 Rust 프로젝트 클론:
   ```bash
   # 원본 프로젝트 (애셋 소스)
   git clone https://github.com/gpgpchan-svg/GP-Chan.git
   git clone https://github.com/gpgpchan-svg/GEMMI-Chan.git

   # Rust 프로젝트
   git clone https://github.com/kirinonakar/Pets.git
   ```

### 📂 권장 폴더 구조

본 프로젝트는 원본 Python 프로젝트의 애셋을 참조하거나 활용할 수 있으므로, 아래와 같은 구조로 배치하는 것을 권장합니다:

```text
Pets/
├── GP-Chan/          # 원본 GP-Chan 저장소
├── GEMMI-Chan/       # 원본 GEMMI-Chan 저장소
└── rust-pets/        # 현재 Rust 프로젝트 (여기에서 실행)
```

3. 애플리케이션 빌드 및 실행:
   ```bash
   cd rust-pets
   cargo run --release
   ```

## 🖱️ 조작 방법

- **마우스 왼쪽 버튼**: 캐릭터를 드래그하여 화면 내 위치를 이동합니다.
- **마우스 오른쪽 버튼**: 컨텍스트 메뉴를 엽니다:
    - 캐릭터 선택 (GP-Chan / GEMMI-Chan)
    - 마우스 따라가기 설정 (On/Off)
    - 프로그램 종료

## 🛠️ 개발 도구

- [Rust](https://www.rust-lang.org/) - 코어 언어
- [eframe/egui](https://github.com/emilk/egui) - GUI 및 렌더링
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - CPU & RAM 모니터링
- [nvml-wrapper](https://github.com/Rust-GPU/nvml-wrapper) - NVIDIA GPU 모니터링

## 📄 라이선스

이 프로젝트는 MIT 라이선스를 따릅니다.
