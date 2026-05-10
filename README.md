# Rust Desktop Pets (GP-Chan & GEMMI-Chan)

<img src="rust-pets/app.png" width="50%" />

Rust로 작성된 고성능 GPU 가속 데스크톱 펫 애플리케이션입니다. GP-Chan과 GEMMI-Chan 캐릭터를 통합하여 지원하며, 실시간 시스템 리소스 모니터링 기능을 제공합니다.

본 프로젝트는 아래의 원본 Python 프로젝트를 **Rust**로 포팅하고 기능을 통합 및 개선한 버전입니다:
- [GP-Chan](https://github.com/gpgpchan-svg/GP-Chan.git)
- [GEMMI-Chan](https://github.com/gpgpchan-svg/GEMMI-Chan.git)

## ✨ 주요 기능

- **캐릭터 전환**: 우클릭 메뉴를 통해 **GP-Chan**과 **GEMMI-Chan** 사이를 실시간으로 전환할 수 있습니다.
- **시스템 리소스 시각화**: CPU, RAM, GPU, VRAM 사용량을 색상별 **그래프 바**로 직관적으로 표시합니다.
- **마우스 상호작용**: 클릭(상호작용), 더블클릭(딱콩), 호버(반응), 스크롤(간지럼) 등 다양한 마우스 동작에 실시간으로 반응합니다.
- **드래그 앤 드롭**: 캐릭터를 잡아 끌어 화면 어디든 이동시킬 수 있으며, 이동 중에는 귀여운 '대롱대롱' 애니메이션이 출력됩니다.
- **커서 추적**: 설정에 따라 캐릭터가 마우스 커서의 위치를 실시간으로 바라보고 추적합니다.
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

- **마우스 왼쪽 버튼**
    - **클릭**: 랜덤 상호작용 애니메이션(인사, 응원 등)을 실행합니다.
    - **더블클릭**: 캐릭터를 꿀밤 때리는('딱콩!') 애니메이션이 실행됩니다.
    - **드래그**: 캐릭터를 잡아 끌어서 화면 내 위치를 자유롭게 이동합니다. (이동 중에는 '대롱대롱' 상태가 됩니다.)
- **마우스 휠 (Scroll)**
    - **스크롤**: 캐릭터 위에서 휠을 굴리면 간지러워하는('아ㅋㅋ 간지러') 반응을 보입니다.
- **마우스 호버 (Hover)**
    - **마우스 올리기**: 마우스 커서를 캐릭터 위로 가져가면 깜짝 놀라거나 인사를 하는 등 즉각적인 반응을 보입니다.
- **마우스 오른쪽 버튼**: 기능 설정 및 액션 선택 메뉴를 엽니다.
    - **캐릭터 변경**: GP-Chan과 GEMMI-Chan 중 원하는 캐릭터로 즉시 전환합니다.
    - **액션 선택**: 수동으로 특정 애니메이션(잠자기, 청소 등)을 명령할 수 있습니다.
    - **마우스 따라오기 (Toggle)**: 캐릭터가 마우스 커서의 위치를 인식하고 추적할지 여부를 결정합니다.
    - **그래프 표시 (Toggle)**: 시스템 리소스 모니터링 그래프 바를 켜거나 끕니다.
    - **종료**: 프로그램을 종료합니다.

## 🌟 특별한 기능

- **실시간 추적**: '마우스 따라오기'가 켜져 있으면 캐릭터가 화면 어디든 커서를 끈질기게 추적합니다.
- **상호작용 반응**: 클릭, 스크롤, 호버 등 다양한 마우스 동작에 실시간으로 반응합니다.
- **타이핑 감지**: 사용자가 키보드를 입력하면 캐릭터가 함께 열심히 타이핑하는 애니메이션을 보여줍니다.
- **상황별 반응**: CPU/RAM 사용량이 높거나 특정 시간대가 되면 캐릭터가 상태에 맞는 대사를 출력합니다.
- **자동 순찰**: 일정 시간 상호작용이 없으면 캐릭터가 화면을 자유롭게 돌아다니며 순찰을 돌기도 합니다.

## 🛠️ 개발 도구

- [Rust](https://www.rust-lang.org/) - 코어 언어
- [eframe/egui](https://github.com/emilk/egui) - GUI 및 렌더링
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - CPU & RAM 모니터링
- [nvml-wrapper](https://github.com/Rust-GPU/nvml-wrapper) - NVIDIA GPU 모니터링

## 📄 라이선스

이 프로젝트는 MIT 라이선스를 따릅니다.
