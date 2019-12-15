class Login extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
            <hf-form class="login" text="who are you ?">
                <input minlength="5" placeholder="passphrase" type="password" name="passphrase" class="passphrase" required/>
                <hf-button content="login" class="submit"></hf-button>
            </hf-form>
        `
    }

    connectedCallback() {
        this.shadowRoot.querySelector(".submit").addEventListener('click', this._login.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector(".submit").removeEventListener('click', this._login);
    }

    _login(e) {
        e.preventDefault();

        let invalid =  this.shadowRoot.querySelector(":invalid");

        if( invalid){
            return;
        }

        let passphrase = this.shadowRoot.querySelector(".passphrase:valid").value;

        console.debug(passphrase)

        fetch("http://localhost:8000/login", {
                method : "POST",
                headers: {
                   cache : "no-cache"
                },
                body : JSON.stringify({
                    passphrase: passphrase
                }),
            })
            .then(res => res.text()) // parse response as JSON with res.json
            .then(response => {

                if(response === "data_required"){
                       e.preventDefault();
                       var event = new CustomEvent('_auth_signup', { 'detail': passphrase });
                       document.dispatchEvent(event);
                }else{
                    var event = new CustomEvent('_auth_login', { 'detail': response });
                    document.dispatchEvent(event);
                }
            })
            .catch(err => {
                console.log({service:"auth", status:"KO", error:err})
                alert("sorry, cannot auth")
            });
    }
}
customElements.define('hf-auth-login', Login);