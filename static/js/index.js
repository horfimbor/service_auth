class Auth extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });

        if(localStorage.getItem("jwt")){
            this.shadowRoot.innerHTML = `<hf-auth-logout></hf-auth-logout>`
        }else{
            this.shadowRoot.innerHTML = `<hf-auth-login></hf-auth-login>`
        }
    }

    connectedCallback() {
        import('http://localhost:8000/js/login.js').then(module => {
          console.log('load login')
        });
        import('http://localhost:8000/js/logout.js').then(module => {
          console.log('load logout')
        });
    }

    _getJwt(){

        // TODO add refresh
        return localStorage.getItem("data")
    }

    _setJwt(jwt){
        localStorage.setItem("jwt", jwt)

        var event = new CustomEvent('login', { 'detail': jwt });
        document.dispatchEvent(event);
    }

    _logout(e) {
        localStorage.removeItem("jwt")

        var event = new CustomEvent('logout');
        document.dispatchEvent(event);
    }
}


customElements.define('hf-auth', Auth);